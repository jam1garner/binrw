use proc_macro2::TokenStream;
use syn::spanned::Spanned;

use crate::parser::TempableField;

use super::super::{
    types::{Assert, CondEndian, EnumErrorMode, Imports, Magic, Map},
    write::FromInput,
    ParseResult, SpannedValue, TrySet,
};
use super::{EnumVariant, StructField, UnitEnumField};

#[derive(Debug)]
pub(crate) enum Input {
    Struct(Struct),
    UnitStruct(Struct),
    Enum(Enum),
    UnitOnlyEnum(UnitOnlyEnum),
}

impl Input {
    pub(crate) fn from_input(
        input: &syn::DeriveInput,
        is_inside_derive: bool,
    ) -> ParseResult<Self> {
        let attrs = &input.attrs;
        match &input.data {
            syn::Data::Struct(st) => {
                let write_struct =
                    Struct::from_input(attrs, st.fields.iter()).map(|mut write_struct| {
                        write_struct.temp_legal = !is_inside_derive;
                        write_struct
                    });

                if matches!(st.fields, syn::Fields::Unit) {
                    write_struct.map(Self::UnitStruct)
                } else {
                    write_struct.map(Self::Struct)
                }
            }
            syn::Data::Enum(en) => {
                let variants = &en.variants;
                if variants.is_empty() {
                    ParseResult::Err(syn::Error::new(
                        input.span(),
                        "null enums are not supported",
                    ))
                } else if variants
                    .iter()
                    .all(|v| matches!(v.fields, syn::Fields::Unit))
                {
                    UnitOnlyEnum::from_input(attrs, variants.iter()).map(Self::UnitOnlyEnum)
                } else {
                    let write_enum =
                        Enum::from_input(attrs, variants.iter()).map(|mut write_enum| {
                            for x in &mut write_enum.variants {
                                if let EnumVariant::Variant { options, .. } = x {
                                    options.temp_legal = !is_inside_derive;
                                }
                            }
                            write_enum
                        });
                    write_enum.map(Self::Enum)
                }
            }
            syn::Data::Union(_) => {
                ParseResult::Err(syn::Error::new(input.span(), "unions are not supported"))
            }
        }
    }

    pub(crate) fn endian(&self) -> &CondEndian {
        match self {
            Input::Struct(s) | Input::UnitStruct(s) => &s.endian,
            Input::Enum(e) => &e.endian,
            Input::UnitOnlyEnum(e) => &e.endian,
        }
    }

    pub(crate) fn imports(&self) -> &Imports {
        match self {
            Input::Struct(s) | Input::UnitStruct(s) => &s.imports,
            Input::Enum(e) => &e.imports,
            Input::UnitOnlyEnum(e) => &e.imports,
        }
    }

    pub(crate) fn is_temp_field(&self, variant_index: usize, index: usize) -> bool {
        match self {
            Input::Struct(s) => s.fields.get(index).map_or(false, TempableField::is_temp),
            Input::Enum(e) => e.variants.get(variant_index).map_or(false, |variant| {
                if let EnumVariant::Variant { options, .. } = variant {
                    options
                        .fields
                        .get(index)
                        .map_or(false, TempableField::is_temp)
                } else {
                    false
                }
            }),
            Input::UnitStruct(_) | Input::UnitOnlyEnum(_) => false,
        }
    }

    pub(crate) fn map(&self) -> &Map {
        match self {
            Input::Struct(s) | Input::UnitStruct(s) => &s.map,
            Input::Enum(e) => &e.map,
            Input::UnitOnlyEnum(e) => &e.map,
        }
    }

    pub(crate) fn magic(&self) -> &Magic {
        match self {
            Input::Struct(s) | Input::UnitStruct(s) => &s.magic,
            Input::Enum(e) => &e.magic,
            Input::UnitOnlyEnum(e) => &e.magic,
        }
    }
}

attr_struct! {
    @write struct_struct

    #[from(StructAttr)]
    #[derive(Clone, Debug, Default)]
    pub(crate) struct Struct {
        pub(crate) temp_legal: bool,
        #[from(Big, Little, IsBig, IsLittle)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap, Repr)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Import, ImportRaw)]
        pub(crate) imports: Imports,
        #[from(Assert)]
        pub(crate) assertions: Vec<Assert>,
        #[from(PreAssert)]
        pub(crate) pre_assertions: Vec<Assert>,
        pub(crate) fields: Vec<StructField>,
    }
}

impl Struct {
    pub(crate) fn is_tuple(&self) -> bool {
        self.fields
            .get(0)
            .map_or(false, |field| field.generated_ident)
    }

    pub(crate) fn iter_permanent_idents(&self) -> impl Iterator<Item = &syn::Ident> + '_ {
        let temp_legal = self.temp_legal;
        self.fields.iter().filter_map(move |field| {
            if temp_legal && field.is_temp() {
                None
            } else {
                Some(&field.ident)
            }
        })
    }

    pub(crate) fn fields_pattern(&self) -> TokenStream {
        let fields = self.iter_permanent_idents();

        if self.is_tuple() {
            quote::quote! {
                (#(ref #fields),*)
            }
        } else {
            quote::quote! {
                { #(ref #fields),* }
            }
        }
    }
}

impl FromInput<StructAttr> for Struct {
    type Field = StructField;

    fn push_field(&mut self, field: Self::Field) -> syn::Result<()> {
        self.fields.push(field);
        Ok(())
    }
}

attr_struct! {
    @write enum_struct

    #[from(EnumAttr)]
    #[derive(Clone, Debug, Default)]
    pub(crate) struct Enum {
        #[from(Big, Little, IsBig, IsLittle)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap, Repr)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Import, ImportRaw)]
        pub(crate) imports: Imports,
        // TODO: Does this make sense? It is not known what properties will
        // exist in order to construct a valid variant. The assertions all get
        // copied and used as if they were applied to each variant in the enum,
        // so the only way this ever works is if every variant contains the same
        // properties being checked by the assertion.
        #[from(Assert)]
        pub(crate) assertions: Vec<Assert>,
        #[from(PreAssert)]
        pub(crate) pre_assertions: Vec<Assert>,
        #[from(ReturnAllErrors, ReturnUnexpectedError)]
        pub(crate) error_mode: EnumErrorMode,
        pub(crate) variants: Vec<EnumVariant>,
    }
}

impl FromInput<EnumAttr> for Enum {
    type Field = EnumVariant;

    fn push_field(&mut self, field: Self::Field) -> syn::Result<()> {
        self.variants.push(field);
        Ok(())
    }
}

attr_struct! {
    @write unit_only_enum

    #[from(UnitEnumAttr)]
    #[derive(Clone, Debug, Default)]
    pub(crate) struct UnitOnlyEnum {
        #[from(Big, Little, IsBig, IsLittle)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Import, ImportRaw)]
        pub(crate) imports: Imports,
        #[from(Repr)]
        pub(crate) repr: Option<SpannedValue<TokenStream>>,
        pub(crate) fields: Vec<UnitEnumField>,
        pub(crate) expected_field_magic: Magic,
    }
}

impl UnitOnlyEnum {
    pub(crate) fn is_magic_enum(&self) -> bool {
        self.expected_field_magic.is_some()
    }
}

impl FromInput<UnitEnumAttr> for UnitOnlyEnum {
    type Field = UnitEnumField;

    fn push_field(&mut self, field: Self::Field) -> syn::Result<()> {
        if let (Some(repr), Some(magic)) = (self.repr.as_ref(), field.magic.as_ref()) {
            let magic_span = magic.span();
            let span = magic_span.join(repr.span()).unwrap_or(magic_span);
            Err(syn::Error::new(
                span,
                "`repr` and `magic` are mutually exclusive",
            ))
        } else {
            let expected_magic = self.expected_field_magic.as_ref();
            match (expected_magic, field.magic.as_ref()) {
                (Some(expected_magic), Some(magic)) => {
                    if expected_magic.kind() != magic.kind() {
                        let magic_span = magic.match_value().span();
                        let span = magic_span
                            .join(expected_magic.match_value().span())
                            .unwrap_or(magic_span);
                        return Err(syn::Error::new(
                            span,
                            format!(
                                "conflicting magic types; expected {}",
                                expected_magic.kind()
                            ),
                        ));
                    }
                }
                (None, Some(_)) => self.expected_field_magic = field.magic.clone(),
                _ => {}
            }

            self.fields.push(field);
            Ok(())
        }
    }

    fn validate(&self) -> syn::Result<()> {
        if self.repr.is_some() || self.is_magic_enum() {
            Ok(())
        } else {
            Err(syn::Error::new(proc_macro2::Span::call_site(), "BinRead on unit-like enums requires either `#[br(repr = ...)]` on the enum or `#[br(magic = ...)]` on at least one variant"))
        }
    }
}
