use super::super::{
    types::{Assert, CondEndian, Condition, Magic, Map, PassedArgs, ReadMode},
    write::{FromAttrs, FromInput},
    FromField, ParseResult, SpannedValue, TrySet,
};

use super::Struct;

use proc_macro2::TokenStream;
use syn::spanned::Spanned;

attr_struct! {
    @write struct_field

    #[from(StructFieldAttr)]
    #[derive(Clone, Debug)]
    pub(crate) struct StructField {
        pub(crate) ident: syn::Ident,
        pub(crate) generated_ident: bool,
        pub(crate) ty: syn::Type,
        #[from(Big, Little, IsBig, IsLittle)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Args, ArgsRaw)]
        pub(crate) args: PassedArgs,
        #[from(Calc, Ignore, ParseWith)]
        pub(crate) read_mode: ReadMode,
        #[from(Count)]
        pub(crate) count: Option<TokenStream>,
        #[from(If)]
        pub(crate) if_cond: Option<Condition>,
        #[from(RestorePosition)]
        pub(crate) restore_position: Option<()>,
        #[from(Temp)]
        pub(crate) temp: Option<()>,
        #[from(Assert)]
        pub(crate) assertions: Vec<Assert>,
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
        matches!(self.read_mode, ReadMode::Normal) && !self.map.is_some()
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
}

impl FromField for StructField {
    type In = syn::Field;

    fn from_field(field: &Self::In, index: usize) -> ParseResult<Self> {
        Self::set_from_attrs(
            Self {
                ident: field
                    .ident
                    .clone()
                    .unwrap_or_else(|| quote::format_ident!("self_{}", index)),
                generated_ident: field.ident.is_none(),
                ty: field.ty.clone(),
                endian: <_>::default(),
                map: <_>::default(),
                magic: <_>::default(),
                args: <_>::default(),
                read_mode: <_>::default(),
                count: <_>::default(),
                if_cond: <_>::default(),
                restore_position: <_>::default(),
                temp: <_>::default(),
                assertions: <_>::default(),
                pad_before: <_>::default(),
                pad_after: <_>::default(),
                align_before: <_>::default(),
                align_after: <_>::default(),
                seek_before: <_>::default(),
                pad_size_to: <_>::default(),
            },
            &field.attrs,
        )
    }
}

attr_struct! {
    @write unit_enum_field

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

impl FromField for UnitEnumField {
    type In = syn::Variant;

    fn from_field(field: &Self::In, _: usize) -> ParseResult<Self> {
        Self::set_from_attrs(
            Self {
                ident: field.ident.clone(),
                magic: <_>::default(),
                pre_assertions: <_>::default(),
            },
            &field.attrs,
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) enum EnumVariant {
    Variant { ident: syn::Ident, options: Struct },
    Unit(UnitEnumField),
}

impl EnumVariant {
    pub(crate) fn ident(&self) -> &syn::Ident {
        match self {
            EnumVariant::Variant { ident, .. } => ident,
            EnumVariant::Unit(field) => &field.ident,
        }
    }
}

impl FromField for EnumVariant {
    type In = syn::Variant;

    fn from_field(variant: &Self::In, index: usize) -> ParseResult<Self> {
        match variant.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                Struct::from_input(&variant.attrs, variant.fields.iter()).map(|options| {
                    Self::Variant {
                        ident: variant.ident.clone(),
                        options,
                    }
                })
            }
            syn::Fields::Unit => UnitEnumField::from_field(variant, index).map(Self::Unit),
        }
    }
}
