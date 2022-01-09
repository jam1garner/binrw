#![allow(dead_code)]
use super::super::{
    types::{Assert, CondEndian, Magic, Map, PassedArgs, WriteMode},
    write::{FromAttrs, FromInput},
    FromField, ParseResult, TrySet,
};

use super::Struct;

use crate::parser::TempableField;
use proc_macro2::TokenStream;

attr_struct! {
    @write struct_field

    #[from(StructFieldAttr)]
    #[derive(Clone, Debug)]
    pub(crate) struct StructField {
        pub(crate) ident: syn::Ident,
        pub(crate) generated_ident: bool,
        pub(crate) ty: syn::Type,
        pub(crate) field: syn::Field,
        #[from(Big, Little, IsBig, IsLittle)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Args, ArgsRaw)]
        pub(crate) args: PassedArgs,
        #[from(Calc, Ignore, WriteWith)]
        pub(crate) write_mode: WriteMode,
        #[from(Count)]
        pub(crate) count: Option<TokenStream>,
        #[from(RestorePosition)]
        pub(crate) restore_position: Option<()>,
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
        // Marker for if binread has marked this field temporary
        pub(crate) binread_temp: bool,
    }
}

impl StructField {
    /// Returns true if this field is read from a parser with an `after_parse`
    /// method.
    pub(crate) fn can_call_after_parse(&self) -> bool {
        matches!(self.write_mode, WriteMode::Normal) && !self.map.is_some()
    }

    /// Returns true if this field is generated using a calculated value instead
    /// of being read from the struct.
    pub(crate) fn generated_value(&self) -> bool {
        matches!(self.write_mode, WriteMode::Calc(_))
    }

    /// Returns true if the field needs `ReadOptions` to be parsed.
    pub(crate) fn needs_options(&self) -> bool {
        !self.generated_value() || self.magic.is_some()
    }

    /// Returns true if the field is actually written.
    pub(crate) fn is_written(&self) -> bool {
        // Non-calc temp fields are not written
        if self.is_temp() && !matches!(self.write_mode, WriteMode::Calc(_)) {
            return false;
        }
        // Ignored fields are not written
        !matches!(self.write_mode, WriteMode::Ignore)
    }
}

impl TempableField for StructField {
    fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    fn is_temp(&self) -> bool {
        self.binread_temp || self.is_temp_for_crossover()
    }

    fn is_temp_for_crossover(&self) -> bool {
        matches!(self.write_mode, WriteMode::Calc(_))
    }

    fn set_crossover_temp(&mut self, temp: bool) {
        self.binread_temp = temp;
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
                field: field.clone(),
                endian: <_>::default(),
                map: <_>::default(),
                magic: <_>::default(),
                args: <_>::default(),
                count: <_>::default(),
                restore_position: <_>::default(),
                write_mode: <_>::default(),
                assertions: <_>::default(),
                pad_before: <_>::default(),
                pad_after: <_>::default(),
                align_before: <_>::default(),
                align_after: <_>::default(),
                seek_before: <_>::default(),
                pad_size_to: <_>::default(),
                keyword_spans: <_>::default(),
                binread_temp: false,
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
}

impl FromField for EnumVariant {
    type In = syn::Variant;

    fn from_field(variant: &Self::In, index: usize) -> ParseResult<Self> {
        match variant.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) => {
                Struct::from_input(&variant.attrs, variant.fields.iter()).map(|options| {
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
