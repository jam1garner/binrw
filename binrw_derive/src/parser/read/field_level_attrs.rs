use super::super::{
    read::{FromAttrs, FromInput},
    types::{Assert, CondEndian, Condition, ErrContext, Magic, Map, PassedArgs, ReadMode},
    FromField, ParseResult, SpannedValue, TrySet,
};

use super::Struct;

use crate::parser::TempableField;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

attr_struct! {
    @read struct_field

    #[from(StructFieldAttr)]
    #[derive(Clone, Debug)]
    pub(crate) struct StructField {
        pub(crate) ident: syn::Ident,
        pub(crate) generated_ident: bool,
        pub(crate) ty: syn::Type,
        pub(crate) field: syn::Field,
        #[from(Big, Little, IsBig, IsLittle)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap, Repr)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Args, ArgsRaw)]
        pub(crate) args: PassedArgs,
        #[from(Calc, Default, Ignore, ParseWith)]
        pub(crate) read_mode: ReadMode,
        #[from(Count)]
        pub(crate) count: Option<TokenStream>,
        #[from(Offset)]
        pub(crate) offset: Option<TokenStream>,
        #[from(OffsetAfter)]
        pub(crate) offset_after: Option<SpannedValue<TokenStream>>,
        #[from(If)]
        pub(crate) if_cond: Option<Condition>,
        #[from(DerefNow, PostProcessNow)]
        pub(crate) deref_now: Option<SpannedValue<()>>,
        #[from(RestorePosition)]
        pub(crate) restore_position: Option<()>,
        #[from(Try)]
        pub(crate) do_try: Option<SpannedValue<()>>,
        #[from(Temp)]
        pub(crate) temp: Option<()>,
        #[from(Assert)]
        pub(crate) assertions: Vec<Assert>,
        #[from(ErrContext)]
        pub(crate) err_context: Option<ErrContext>,
        #[from(PadBefore)]
        pub(crate) pad_before: Option<TokenStream>,
        #[from(PadAfter)]
        pub(crate) pad_after: Option<TokenStream>,
        #[from(AlignBefore)]
        pub(crate) align_before: Option<TokenStream>,
        #[from(AlignAfter)]
        pub(crate) align_after: Option<TokenStream>,
        #[from(SeekBefore)]
        pub(crate) seek_before: Option<TokenStream>,
        #[from(PadSizeTo)]
        pub(crate) pad_size_to: Option<TokenStream>,
    }
}

impl StructField {
    /// Returns true if this field is read from a parser with an `after_parse`
    /// method.
    pub(crate) fn can_call_after_parse(&self) -> bool {
        matches!(self.read_mode, ReadMode::Normal) && self.map.is_none()
    }

    /// Returns true if the code generator should emit `BinRead::after_parse()`
    /// after all fields have been read.
    pub(crate) fn should_use_after_parse(&self) -> bool {
        self.deref_now.is_none() && self.map.is_none()
    }

    /// Returns true if this field is generated using a calculated value instead
    /// of a parser.
    pub(crate) fn generated_value(&self) -> bool {
        matches!(self.read_mode, ReadMode::Calc(_) | ReadMode::Default)
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
            && matches!(self.read_mode, ReadMode::Normal)
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

    fn validate(&self) -> syn::Result<()> {
        if let (Some(offset_after), Some(deref_now)) = (&self.offset_after, &self.deref_now) {
            let offset_after_span = offset_after.span();
            let span = offset_after_span
                .join(deref_now.span())
                .unwrap_or(offset_after_span);
            Err(syn::Error::new(
                span,
                "`deref_now` and `offset_after` are mutually exclusive",
            ))
        } else if self.do_try.is_some() && self.generated_value() {
            //TODO: join with span of read mode somehow
            let span = self.do_try.as_ref().unwrap().span();
            Err(syn::Error::new(
                span,
                "`try` is incompatible with `default` and `calc`",
            ))
        } else {
            Ok(())
        }
    }
}

impl TempableField for StructField {
    fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    fn is_temp(&self) -> bool {
        self.is_temp_for_crossover()
    }

    fn is_temp_for_crossover(&self) -> bool {
        self.temp.is_some()
    }

    fn set_crossover_temp(&mut self, temp: bool) {
        self.temp = temp.then(|| ());
    }
}

impl FromField for StructField {
    type In = syn::Field;

    fn from_field(field: &Self::In, index: usize) -> ParseResult<Self> {
        let result = Self::set_from_attrs(
            Self {
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
            },
            &field.attrs,
        );

        match result {
            ParseResult::Ok(this) => {
                if let Err(error) = this.validate() {
                    ParseResult::Partial(this, error)
                } else {
                    ParseResult::Ok(this)
                }
            }
            ParseResult::Partial(this, mut parse_error) => {
                if let Err(error) = this.validate() {
                    parse_error.combine(error);
                }
                ParseResult::Partial(this, parse_error)
            }
            ParseResult::Err(error) => ParseResult::Err(error),
        }
    }
}

attr_struct! {
    @read unit_enum_field

    #[from(UnitEnumFieldAttr)]
    #[derive(Clone, Debug)]
    pub(crate) struct UnitEnumField {
        pub(crate) ident: syn::Ident,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(PreAssert)]
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

    fn from_field(field: &Self::In, _: usize) -> ParseResult<Self> {
        Self::set_from_attrs(
            Self {
                ident: field.ident.clone(),
                magic: <_>::default(),
                pre_assertions: <_>::default(),
                keyword_spans: <_>::default(),
            },
            &field.attrs,
        )
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

    fn from_field(variant: &Self::In, index: usize) -> ParseResult<Self> {
        match variant.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                Struct::from_input(None, &variant.attrs, variant.fields.iter()).map(|options| {
                    Self::Variant {
                        ident: variant.ident.clone(),
                        options: Box::new(options),
                    }
                })
            }
            syn::Fields::Unit => UnitEnumField::from_field(variant, index).map(Self::Unit),
        }
    }
}
