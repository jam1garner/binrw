use super::{
    attr_struct,
    top_level_attrs::StructAttr,
    types::{Assert, Bound, CondEndian, Condition, ErrContext, FieldMode, Magic, Map, PassedArgs},
    FromAttrs, FromField, FromInput, ParseResult, SpannedValue, Struct, TrySet,
};
use crate::{binrw::Options, combine_error};
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
        #[from(RW:MapStream)]
        pub(crate) map_stream: Option<TokenStream>,
        #[from(RW:Magic)]
        pub(crate) magic: Magic,
        #[from(RW:Args, RW:ArgsRaw)]
        pub(crate) args: PassedArgs,
        #[from(RW:Calc, RW:TryCalc, RO:Default, RW:Ignore, RO:ParseWith, WO:WriteWith)]
        pub(crate) field_mode: FieldMode,
        #[from(RO:Count)]
        pub(crate) count: Option<TokenStream>,
        #[from(RO:Offset)]
        pub(crate) offset: Option<TokenStream>,
        #[from(RW:If)]
        pub(crate) if_cond: Option<Condition>,
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
        #[from(RO:Debug)] // TODO is this really RO?
        pub(crate) debug: Option<()>,
        #[from(RW:Bound)]
        pub(crate) bound: Bound,
    }
}

impl StructField {
    /// Returns true if this field is generated using a calculated value instead
    /// of a parser.
    pub(crate) fn generated_value(&self) -> bool {
        matches!(
            self.field_mode,
            FieldMode::TryCalc(_) | FieldMode::Calc(_) | FieldMode::Default
        )
    }

    /// Returns true if the field is handled as a temporary variable instead of
    /// an actual field.
    pub(crate) fn is_temp(&self, for_write: bool) -> bool {
        (for_write && matches!(self.field_mode, FieldMode::TryCalc(_) | FieldMode::Calc(_)))
            || self.temp.is_some()
    }

    /// Returns true if the field is actually written.
    pub(crate) fn is_written(&self) -> bool {
        !matches!(self.field_mode, FieldMode::Default)
    }

    /// Returns true if the field requires arguments.
    pub(crate) fn needs_args(&self) -> bool {
        self.args.is_some() || self.count.is_some() || self.offset.is_some()
    }

    /// Returns true if the field overrides endianness.
    pub(crate) fn needs_endian(&self) -> bool {
        !matches!(self.endian, CondEndian::Inherited)
    }

    /// Returns true if the field is using shorthand directives that are
    /// converted into named arguments.
    pub(crate) fn has_named_arg_directives(&self) -> bool {
        self.count.is_some() || self.offset.is_some()
    }

    /// Returns true if the only field-level attributes are asserts
    pub(crate) fn has_no_attrs(&self) -> bool {
        macro_rules! all_fields_none {
            ($($field:ident),*) => {
                $(
                    self.$field.is_none() &&
                )*

                true
            }
        }

        matches!(self.endian, CondEndian::Inherited)
            && matches!(self.map, Map::None)
            && matches!(self.args, PassedArgs::None)
            && matches!(self.field_mode, FieldMode::Normal)
            && all_fields_none!(
                count,
                offset,
                if_cond,
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

    /// Forces the field to be treated as a temporary variable even if it was
    /// not explicitly specified by a directive.
    ///
    /// This is used to ensure that, when combining read and write on a single
    /// type, a field specified as temporary on one side is treated as a
    /// temporary on both sides.
    pub(crate) fn force_temp(&mut self) {
        self.temp = Some(());
    }

    fn validate(&self, options: Options) -> syn::Result<()> {
        let mut all_errors = None::<syn::Error>;

        if self.do_try.is_some() && self.generated_value() {
            //TODO: join with span of read mode somehow
            let span = self.do_try.as_ref().unwrap().span();
            combine_error(
                &mut all_errors,
                syn::Error::new(
                    span,
                    "`try` is incompatible with `default`, `calc`, and `try_calc`",
                ),
            );
        }

        if !options.write
            && matches!(self.field_mode, FieldMode::TryCalc(_) | FieldMode::Calc(_))
            && self.args.is_some()
        {
            // TODO: Correct span (args + calc keywords)
            combine_error(
                &mut all_errors,
                syn::Error::new(
                    self.field.span(),
                    "`args` is incompatible with `calc` and `try_calc`",
                ),
            );
        }

        if self.has_named_arg_directives()
            && !matches!(self.args, PassedArgs::None | PassedArgs::Named(..))
        {
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

            for (used, name) in [
                (self.count.is_some(), "count"),
                (self.offset.is_some(), "offset"),
            ] {
                if used {
                    combine_error(&mut all_errors, syn::Error::new(
                        span,
                        format!("`{name}` can only be used with named args; did you mean `args {{ inner: {repr} }}`?")
                    ));
                }
            }
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
            map_stream: <_>::default(),
            magic: <_>::default(),
            args: <_>::default(),
            field_mode: <_>::default(),
            count: <_>::default(),
            offset: <_>::default(),
            if_cond: <_>::default(),
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
            #[cfg(feature = "verbose-backtrace")]
            keyword_spans: <_>::default(),
            err_context: <_>::default(),
            debug: <_>::default(),
            bound: <_>::default(),
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
            #[cfg(feature = "verbose-backtrace")]
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
