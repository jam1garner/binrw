use super::{
    attr_struct,
    top_level_attrs::StructAttr,
    types::{Assert, CondEndian, Condition, ErrContext, FieldMode, Magic, Map, PassedArgs},
    FromAttrs, FromField, FromInput, ParseResult, SpannedValue, Struct, TrySet,
};
use crate::{combine_error, Options};
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

attr_struct! {
    #[from(StructFieldAttr)]
    #[derive(Clone, Debug)]
    pub(crate) struct StructField {
        pub(crate) ident: syn::Ident,
        pub(crate) generated_ident: bool,
        pub(crate) ty: syn::Type,
        pub(crate) field: syn::Field,
        #[from(RW:Big, RW:Little, RW:IsBig, RW:IsLittle)]
        pub(crate) endian: CondEndian,
        #[from(RW:Map, RW:TryMap, RW:Repr)]
        pub(crate) map: Map,
        #[from(RW:Magic)]
        pub(crate) magic: Magic,
        #[from(RW:Args, RW:ArgsRaw)]
        pub(crate) args: PassedArgs,
        #[from(RW:Calc, RO:Default, RW:Ignore, RO:ParseWith, WO:WriteWith)]
        pub(crate) read_mode: FieldMode,
        #[from(RO:Count)]
        pub(crate) count: Option<TokenStream>,
        #[from(RO:Offset)]
        pub(crate) offset: Option<TokenStream>,
        #[from(RO:OffsetAfter)]
        pub(crate) offset_after: Option<SpannedValue<TokenStream>>,
        #[from(RO:If)]
        pub(crate) if_cond: Option<Condition>,
        #[from(RO:DerefNow, RO:PostProcessNow)]
        pub(crate) deref_now: Option<SpannedValue<()>>,
        #[from(RW:RestorePosition)]
        pub(crate) restore_position: Option<()>,
        #[from(RO:Try)]
        pub(crate) do_try: Option<SpannedValue<()>>,
        #[from(RO:Temp)]
        pub(crate) temp: Option<()>,
        #[from(RW:Assert)]
        pub(crate) assertions: Vec<Assert>,
        #[from(RO:ErrContext)]
        pub(crate) err_context: Option<ErrContext>,
        #[from(RW:PadBefore)]
        pub(crate) pad_before: Option<TokenStream>,
        #[from(RW:PadAfter)]
        pub(crate) pad_after: Option<TokenStream>,
        #[from(RW:AlignBefore)]
        pub(crate) align_before: Option<TokenStream>,
        #[from(RW:AlignAfter)]
        pub(crate) align_after: Option<TokenStream>,
        #[from(RW:SeekBefore)]
        pub(crate) seek_before: Option<TokenStream>,
        #[from(RW:PadSizeTo)]
        pub(crate) pad_size_to: Option<TokenStream>,
    }
}

impl StructField {
    /// Returns true if this field is read from a parser with an `after_parse`
    /// method.
    pub(crate) fn can_call_after_parse(&self) -> bool {
        matches!(self.read_mode, FieldMode::Normal) && self.map.is_none()
    }

    /// Returns true if the code generator should emit `BinRead::after_parse()`
    /// after all fields have been read.
    pub(crate) fn should_use_after_parse(&self) -> bool {
        self.deref_now.is_none() && self.map.is_none()
    }

    /// Returns true if this field is generated using a calculated value instead
    /// of a parser.
    pub(crate) fn generated_value(&self) -> bool {
        matches!(self.read_mode, FieldMode::Calc(_) | FieldMode::Default)
    }

    pub(crate) fn is_temp(&self, for_write: bool) -> bool {
        (for_write && matches!(self.read_mode, FieldMode::Calc(_))) || self.temp.is_some()
    }

    /// Returns true if the field is actually written.
    pub(crate) fn is_written(&self) -> bool {
        !matches!(self.read_mode, FieldMode::Default)
    }

    /// Returns true if the field needs `ReadOptions` to be parsed.
    pub(crate) fn needs_options(&self) -> bool {
        !self.generated_value() || self.magic.is_some()
    }

    /// Returns true if the only field-level attributes are asserts
    pub(crate) fn has_no_attrs(&self) -> bool {
        macro_rules! all_fields_none {
            ($($field:ident),*) => {
                $(
                    matches!(self.$field, None) &&
                )*

                true
            }
        }

        matches!(self.endian, CondEndian::Inherited)
            && matches!(self.map, Map::None)
            && matches!(self.args, PassedArgs::None)
            && matches!(self.read_mode, FieldMode::Normal)
            && all_fields_none!(
                count,
                offset,
                offset_after,
                if_cond,
                deref_now,
                restore_position,
                do_try,
                temp,
                pad_before,
                pad_after,
                align_before,
                align_after,
                seek_before,
                pad_size_to,
                magic
            )
    }

    pub(crate) fn force_temp(&mut self) {
        self.temp = Some(());
    }

    fn validate(&self, _: Options) -> syn::Result<()> {
        let mut all_errors = None::<syn::Error>;

        if let (Some(offset_after), Some(deref_now)) = (&self.offset_after, &self.deref_now) {
            let offset_after_span = offset_after.span();
            let span = offset_after_span
                .join(deref_now.span())
                .unwrap_or(offset_after_span);
            combine_error(
                &mut all_errors,
                syn::Error::new(
                    span,
                    "`deref_now` and `offset_after` are mutually exclusive",
                ),
            );
        }

        if self.do_try.is_some() && self.generated_value() {
            //TODO: join with span of read mode somehow
            let span = self.do_try.as_ref().unwrap().span();
            combine_error(
                &mut all_errors,
                syn::Error::new(span, "`try` is incompatible with `default` and `calc`"),
            );
        }

        if matches!(self.read_mode, FieldMode::Calc(_)) && self.args.is_some() {
            // TODO: Correct span (args + calc keywords)
            combine_error(
                &mut all_errors,
                syn::Error::new(self.field.span(), "`args` is incompatible with `calc`"),
            );
        }

        if self.count.is_some() && !matches!(self.args, PassedArgs::None | PassedArgs::Named(..)) {
            let (span, repr) = match &self.args {
                PassedArgs::Named(_) | PassedArgs::None => unreachable!(),
                PassedArgs::List(list) => (
                    list.span(),
                    format!(
                        "({},{})",
                        list.first().map_or_else(<_>::default, ToString::to_string),
                        if list.len() > 1 { " ..." } else { "" }
                    ),
                ),
                PassedArgs::Tuple(raw) => (raw.span(), raw.to_string()),
            };

            combine_error(&mut all_errors, syn::Error::new(
                span,
                format!("`count` can only be used with named args; did you mean `args {{ inner: {} }}`?", repr)
            ));
        }

        if let Some(error) = all_errors {
            Err(error)
        } else {
            Ok(())
        }
    }
}

impl FromField for StructField {
    type In = syn::Field;

    fn from_field(field: &Self::In, index: usize, options: Options) -> ParseResult<Self> {
        let this = Self {
            ident: field
                .ident
                .clone()
                .unwrap_or_else(|| quote::format_ident!("self_{}", index)),
            generated_ident: field.ident.is_none(),
            ty: field.ty.clone(),
            field: field.clone(),
            endian: <_>::default(),
            map: <_>::default(),
            magic: <_>::default(),
            args: <_>::default(),
            read_mode: <_>::default(),
            count: <_>::default(),
            offset: <_>::default(),
            offset_after: <_>::default(),
            if_cond: <_>::default(),
            deref_now: <_>::default(),
            restore_position: <_>::default(),
            do_try: <_>::default(),
            temp: <_>::default(),
            assertions: <_>::default(),
            pad_before: <_>::default(),
            pad_after: <_>::default(),
            align_before: <_>::default(),
            align_after: <_>::default(),
            seek_before: <_>::default(),
            pad_size_to: <_>::default(),
            keyword_spans: <_>::default(),
            err_context: <_>::default(),
        };

        let result = if options.write {
            <Self as FromAttrs<StructFieldAttr<true>>>::set_from_attrs(this, &field.attrs, options)
        } else {
            <Self as FromAttrs<StructFieldAttr<false>>>::set_from_attrs(this, &field.attrs, options)
        };

        match result {
            ParseResult::Ok(this) => {
                if let Err(error) = this.validate(options) {
                    ParseResult::Partial(this, error)
                } else {
                    ParseResult::Ok(this)
                }
            }
            ParseResult::Partial(this, mut parse_error) => {
                if let Err(error) = this.validate(options) {
                    parse_error.combine(error);
                }
                ParseResult::Partial(this, parse_error)
            }
            ParseResult::Err(error) => ParseResult::Err(error),
        }
    }
}

attr_struct! {
    #[from(UnitEnumFieldAttr)]
    #[derive(Clone, Debug)]
    pub(crate) struct UnitEnumField {
        pub(crate) ident: syn::Ident,
        #[from(RW:Magic)]
        pub(crate) magic: Magic,
        #[from(RO:PreAssert)]
        pub(crate) pre_assertions: Vec<Assert>,
    }
}

impl From<UnitEnumField> for Struct {
    fn from(value: UnitEnumField) -> Self {
        Self {
            magic: value.magic,
            pre_assertions: value.pre_assertions,
            ..<_>::default()
        }
    }
}

impl FromField for UnitEnumField {
    type In = syn::Variant;

    fn from_field(field: &Self::In, _: usize, options: Options) -> ParseResult<Self> {
        let this = Self {
            ident: field.ident.clone(),
            magic: <_>::default(),
            pre_assertions: <_>::default(),
            keyword_spans: <_>::default(),
        };

        if options.write {
            <Self as FromAttrs<UnitEnumFieldAttr<true>>>::set_from_attrs(
                this,
                &field.attrs,
                options,
            )
        } else {
            <Self as FromAttrs<UnitEnumFieldAttr<false>>>::set_from_attrs(
                this,
                &field.attrs,
                options,
            )
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum EnumVariant {
    Variant {
        ident: syn::Ident,
        options: Box<Struct>,
    },
    Unit(UnitEnumField),
}

impl EnumVariant {
    pub(crate) fn ident(&self) -> &syn::Ident {
        match self {
            EnumVariant::Variant { ident, .. } => ident,
            EnumVariant::Unit(field) => &field.ident,
        }
    }

    pub(crate) fn has_no_attrs(&self) -> bool {
        match self {
            Self::Variant { options, .. } => options.has_no_attrs(),
            Self::Unit(_) => true,
        }
    }
}

impl From<EnumVariant> for Struct {
    fn from(value: EnumVariant) -> Self {
        match value {
            EnumVariant::Variant { options, .. } => *options,
            EnumVariant::Unit(options) => options.into(),
        }
    }
}

impl FromField for EnumVariant {
    type In = syn::Variant;

    fn from_field(variant: &Self::In, index: usize, options: Options) -> ParseResult<Self> {
        match variant.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => if options.write {
                <Struct as FromInput<StructAttr<true>>>::from_input(
                    &variant.attrs,
                    variant.fields.iter(),
                    options,
                )
            } else {
                <Struct as FromInput<StructAttr<false>>>::from_input(
                    &variant.attrs,
                    variant.fields.iter(),
                    options,
                )
            }
            .map(|options| Self::Variant {
                ident: variant.ident.clone(),
                options: Box::new(options),
            }),
            syn::Fields::Unit => UnitEnumField::from_field(variant, index, options).map(Self::Unit),
        }
    }
}
