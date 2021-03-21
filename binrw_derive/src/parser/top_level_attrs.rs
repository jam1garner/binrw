use super::{
    types::{Assert, CondEndian, EnumErrorMode, Imports, Magic, Map},
    EnumVariant, FromInput, ParseResult, SpannedValue, StructField, TrySet, UnitEnumField,
};
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

pub(crate) enum Input {
    Struct(Struct),
    UnitStruct(Struct),
    Enum(Enum),
    UnitOnlyEnum(UnitOnlyEnum),
}

impl Input {
    pub(crate) fn from_input(input: &syn::DeriveInput) -> ParseResult<Self> {
        let attrs = &input.attrs;
        match &input.data {
            syn::Data::Struct(st) => {
                if matches!(st.fields, syn::Fields::Unit) {
                    Struct::from_input(attrs, st.fields.iter()).map(Self::UnitStruct)
                } else {
                    Struct::from_input(attrs, st.fields.iter()).map(Self::Struct)
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
                    Enum::from_input(attrs, variants.iter()).map(Self::Enum)
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
            Input::Struct(s) => s.fields.get(index).map_or(false, |field| field.temp),
            Input::Enum(e) => e.variants.get(variant_index).map_or(false, |variant| {
                if let EnumVariant::Variant { options, .. } = variant {
                    options.fields.get(index).map_or(false, |field| field.temp)
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

    pub(crate) fn pre_assertions(&self) -> &Vec<Assert> {
        match self {
            Input::Struct(s) | Input::UnitStruct(s) => &s.pre_assertions,
            Input::Enum(e) => &e.pre_assertions,
            Input::UnitOnlyEnum(_) => unimplemented!("`Input::pre_assert()` called on unit enum"),
        }
    }
}

attr_struct! {
    #[from(StructAttr)]
    #[derive(Clone, Debug, Default)]
    pub(crate) struct Struct {
        #[from(Big, Little)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Import, ImportTuple)]
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
        self.fields
            .iter()
            .filter_map(|field| if field.temp { None } else { Some(&field.ident) })
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
        #[from(Big, Little)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Import, ImportTuple)]
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

impl Enum {
    pub(crate) fn with_variant(&self, variant: &EnumVariant) -> Self {
        let mut out = self.clone();

        match variant {
            EnumVariant::Variant { options, .. } => {
                if options.endian.is_some() {
                    out.endian = options.endian.clone();
                }

                if options.magic.is_some() {
                    out.magic = options.magic.clone();
                }

                out.pre_assertions
                    .extend_from_slice(&options.pre_assertions);
                out.assertions.extend_from_slice(&options.assertions);
            }

            EnumVariant::Unit(options) => {
                if options.magic.is_some() {
                    out.magic = options.magic.clone();
                }

                out.pre_assertions
                    .extend_from_slice(&options.pre_assertions);
            }
        }

        out
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
    #[from(UnitEnumAttr)]
    #[derive(Clone, Debug, Default)]
    pub(crate) struct UnitOnlyEnum {
        #[from(Big, Little)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Import, ImportTuple)]
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
