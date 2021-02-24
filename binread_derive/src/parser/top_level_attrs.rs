use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use super::{EnumVariant, FromInput, SpannedValue, StructField, TrySet, UnitEnumField, types::{Assert, CondEndian, EnumErrorMode, Imports, Magic, Map}};

#[derive(Clone, Debug)]
pub(crate) enum Input {
    Struct(Struct),
    UnitStruct(Struct),
    Enum(Enum),
    UnitOnlyEnum(UnitOnlyEnum),
}

impl Input {
    pub(crate) fn from_input(input: &syn::DeriveInput) -> syn::Result<Self> {
        let attrs = &input.attrs;
        match &input.data {
            syn::Data::Struct(st) => {
                if matches!(st.fields, syn::Fields::Unit) {
                    Ok(Self::UnitStruct(Struct::from_input(attrs, st.fields.iter())?))
                } else {
                    Ok(Self::Struct(Struct::from_input(attrs, st.fields.iter())?))
                }
            },
            syn::Data::Enum(en) => {
                let variants = &en.variants;
                if variants.is_empty() {
                    Err(syn::Error::new(input.span(), "null enums are not supported"))
                } else if variants.iter().all(|v| matches!(v.fields, syn::Fields::Unit)) {
                    Ok(Self::UnitOnlyEnum(UnitOnlyEnum::from_input(attrs, variants.iter())?))
                } else {
                    Ok(Self::Enum(Enum::from_input(attrs, variants.iter())?))
                }
            },
            syn::Data::Union(_) =>
                Err(syn::Error::new(input.span(), "unions are not supported"))
        }
    }

    pub(crate) fn endian(&self) -> &CondEndian {
        match self {
            Input::Struct(s)
            | Input::UnitStruct(s) => &s.endian,
            Input::Enum(e) => &e.endian,
            Input::UnitOnlyEnum(e) => &e.endian,
        }
    }

    pub(crate) fn imports(&self) -> &Imports {
        match self {
            Input::Struct(s)
            | Input::UnitStruct(s) => &s.import,
            Input::Enum(e) => &e.import,
            Input::UnitOnlyEnum(_) => &Imports::None,
        }
    }

    pub(crate) fn map(&self) -> &Map {
        match self {
            Input::Struct(s)
            | Input::UnitStruct(s) => &s.map,
            Input::Enum(e) => &e.map,
            Input::UnitOnlyEnum(e) => &e.map,
        }
    }

    pub(crate) fn magic(&self) -> &Magic {
        match self {
            Input::Struct(s)
            | Input::UnitStruct(s) => &s.magic,
            Input::Enum(e) => &e.magic,
            Input::UnitOnlyEnum(e) => &e.magic,
        }
    }

    pub(crate) fn pre_assert(&self) -> &Vec<Assert> {
        match self {
            Input::Struct(s)
            | Input::UnitStruct(s) => &s.pre_assert,
            Input::Enum(e) => &e.pre_assert,
            Input::UnitOnlyEnum(_) => panic!("pre_assert on unit enum"),
        }
    }
}

attr_struct! {
    #[from(StructAttr)]
    #[derive(Clone, Debug, Default)]
    pub(crate) struct Struct {
        #[from(Big, Little)]
        pub endian: CondEndian,
        #[from(Map, TryMap)]
        pub map: Map,
        #[from(Magic)]
        pub magic: Magic,
        #[from(Import, ImportTuple)]
        pub import: Imports,
        #[from(Assert)]
        pub assert: Vec<Assert>,
        // TODO: Are Magic and PreAssert conflicting preconditions? Is PreAssert
        // only for enum variants?
        #[from(PreAssert)]
        pub pre_assert: Vec<Assert>,
        pub fields: Vec<StructField>,
    }
}

impl Struct {
    pub(crate) fn is_tuple(&self) -> bool {
        self.fields.get(0).map_or(false, |field| field.ident.is_none())
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
        pub endian: CondEndian,
        #[from(Map, TryMap)]
        pub map: Map,
        #[from(Magic)]
        pub magic: Magic,
        #[from(Import, ImportTuple)]
        pub import: Imports,
        #[from(Assert)]
        pub assert: Vec<Assert>,
        #[from(PreAssert)]
        pub pre_assert: Vec<Assert>,
        #[from(ReturnAllErrors, ReturnUnexpectedError)]
        pub error_mode: EnumErrorMode,
        pub variants: Vec<EnumVariant>,
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

                out.pre_assert.extend_from_slice(&options.pre_assert);
                out.assert.extend_from_slice(&options.assert);
            },

            EnumVariant::Unit(options) => {
                if options.magic.is_some() {
                    out.magic = options.magic.clone();
                }

                out.pre_assert.extend_from_slice(&options.pre_assert);
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
        pub endian: CondEndian,
        #[from(Map, TryMap)]
        pub map: Map,
        #[from(Magic)]
        pub magic: Magic,
        #[from(Repr)]
        pub repr: Option<SpannedValue<TokenStream>>,
        pub fields: Vec<UnitEnumField>,
    }
}

impl UnitOnlyEnum {
    pub(crate) fn is_magic_enum(&self) -> bool {
        self.fields.get(0).map_or(false, |field| field.magic.is_some())
    }
}

impl FromInput<UnitEnumAttr> for UnitOnlyEnum {
    type Field = UnitEnumField;

    fn push_field(&mut self, field: Self::Field) -> syn::Result<()> {
        if let (Some(repr), Some(magic)) = (self.repr.as_ref(), field.magic.as_ref()) {
            let magic_span = magic.span();
            let span = magic_span.join(repr.span()).unwrap_or(magic_span);
            Err(syn::Error::new(span, "`repr` and `magic` are mutually exclusive"))
        } else {
            let expected_magic = self.fields.get(0).unwrap_or(&field).magic.as_ref();
            if let (Some(expected_magic), Some(magic)) = (expected_magic, field.magic.as_ref()) {
                if expected_magic.0 != magic.0 {
                    let magic_span = magic.1.span();
                    let span = magic_span.join(expected_magic.1.span()).unwrap_or(magic_span);
                    return Err(syn::Error::new(span, format!("conflicting magic types; expected {}", expected_magic.0)));
                }
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
