use super::super::{
    read::FromInput,
    types::{Assert, CondEndian, EnumErrorMode, Imports, Magic, Map},
    ParseResult, SpannedValue, TrySet,
};
use super::{EnumVariant, StructField, UnitEnumField};

use proc_macro2::TokenStream;
use syn::spanned::Spanned;

/// The parsed representation of binrw attributes on a data structure.
pub(crate) enum Input {
    /// A normal or tuple struct.
    Struct(Struct),
    /// A unit struct.
    UnitStruct(Struct),
    /// An enum with at least one data variant.
    Enum(Enum),
    /// An enum containing only unit variants.
    UnitOnlyEnum(UnitOnlyEnum),
}

impl Input {
    /// Tries parsing the binrw attributes on a data structure.
    pub(crate) fn from_input(
        input: &syn::DeriveInput,
        is_inside_derive: bool,
        for_write: bool,
    ) -> ParseResult<Self> {
        let attrs = &input.attrs;
        let ident = Some(&input.ident);
        match &input.data {
            syn::Data::Struct(st) => {
                let read_struct = Struct::from_input(ident, attrs, st.fields.iter(), for_write)
                    .map(|mut read_struct| {
                        read_struct.temp_legal = !is_inside_derive;
                        read_struct.for_write = for_write;
                        read_struct
                    });

                if matches!(st.fields, syn::Fields::Unit) {
                    read_struct.map(Self::UnitStruct)
                } else {
                    read_struct.map(Self::Struct)
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
                    UnitOnlyEnum::from_input(ident, attrs, variants.iter(), for_write)
                        .map(Self::UnitOnlyEnum)
                } else {
                    let read_enum = Enum::from_input(ident, attrs, variants.iter(), for_write).map(
                        |mut read_enum| {
                            for x in &mut read_enum.variants {
                                if let EnumVariant::Variant { options, .. } = x {
                                    options.temp_legal = !is_inside_derive;
                                    options.for_write = for_write;
                                }
                            }
                            read_enum
                        },
                    );
                    read_enum.map(Self::Enum)
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
            Input::Struct(s) => s
                .fields
                .get(index)
                .map_or(false, |field| field.is_temp(s.for_write)),
            Input::Enum(e) => e.variants.get(variant_index).map_or(false, |variant| {
                if let EnumVariant::Variant { options, .. } = variant {
                    options
                        .fields
                        .get(index)
                        .map_or(false, |field| field.is_temp(options.for_write))
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

    pub(crate) fn pre_assertions(&self) -> &[Assert] {
        match self {
            Input::Struct(s) | Input::UnitStruct(s) => &s.pre_assertions,
            Input::Enum(e) => &e.pre_assertions,
            Input::UnitOnlyEnum(_) => &[],
        }
    }

    pub(crate) fn assertions(&self) -> &[Assert] {
        match self {
            Input::Struct(s) | Input::UnitStruct(s) => &s.assertions,
            Input::Enum(e) => &e.assertions,
            Input::UnitOnlyEnum(_) => &[],
        }
    }
}

attr_struct! {
    #[from(StructAttr)]
    #[derive(Clone, Debug, Default)]
    pub(crate) struct Struct {
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
        /// If `true`, the struct itself can be modified.
        pub(crate) temp_legal: bool,
        pub(crate) for_write: bool,
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
            if temp_legal && field.is_temp(self.for_write) {
                None
            } else {
                Some(&field.ident)
            }
        })
    }

    pub(crate) fn has_no_attrs(&self) -> bool {
        matches!(self.endian, CondEndian::Inherited)
            && matches!(self.map, Map::None)
            && matches!(self.magic, None)
            && matches!(self.imports, Imports::None)
            && self.fields.iter().all(StructField::has_no_attrs)
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
    #[from(EnumAttr)]
    #[derive(Clone, Debug, Default)]
    pub(crate) struct Enum {
        pub(crate) ident: Option<syn::Ident>,
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

    fn set_ident(&mut self, ident: &syn::Ident) {
        self.ident = Some(ident.clone());
    }
}

attr_struct! {
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
        pub(crate) is_magic_enum: bool,
    }
}

impl UnitOnlyEnum {
    pub(crate) fn is_magic_enum(&self) -> bool {
        self.is_magic_enum
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
            self.is_magic_enum |= field.magic.is_some();
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
