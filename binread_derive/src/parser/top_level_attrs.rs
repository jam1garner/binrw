use proc_macro2::TokenStream;
use super::{EnumVariant, FromInput, TrySet, StructField, UnitEnumField, types::{Assert, CondEndian, EnumErrorMode, Imports, Magic, Map}};

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
                    Err(syn::Error::new(en.enum_token.span, "null enums are not supported"))
                } else if variants.iter().all(|v| matches!(v.fields, syn::Fields::Unit)) {
                    Ok(Self::UnitOnlyEnum(UnitOnlyEnum::from_input(attrs, variants.iter())?))
                } else {
                    Ok(Self::Enum(Enum::from_input(attrs, variants.iter())?))
                }
            },
            syn::Data::Union(union) =>
                Err(syn::Error::new(union.union_token.span, "unions are not supported"))
        }
    }

    #[deprecated]
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

    #[deprecated]
    pub(crate) fn map(&self) -> &Map {
        match self {
            Input::Struct(s)
            | Input::UnitStruct(s) => &s.map,
            Input::Enum(e) => &e.map,
            Input::UnitOnlyEnum(e) => &e.map,
        }
    }

    #[deprecated]
    pub(crate) fn magic(&self) -> &Magic {
        match self {
            Input::Struct(s)
            | Input::UnitStruct(s) => &s.magic,
            Input::Enum(e) => &e.magic,
            Input::UnitOnlyEnum(e) => &e.magic,
        }
    }

    #[deprecated]
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
        pub repr: Option<TokenStream>,
        pub fields: Vec<UnitEnumField>,
    }
}

impl UnitOnlyEnum {
    pub(crate) fn is_magic_enum(&self) -> bool {
        self.fields.get(0).map_or(false, |field| field.magic.is_some())
    }

    pub(crate) fn is_repr_enum(&self) -> bool {
        self.repr.is_some()
    }
}

impl FromInput<UnitEnumAttr> for UnitOnlyEnum {
    type Field = UnitEnumField;

    fn push_field(&mut self, field: Self::Field) -> syn::Result<()> {
        if self.is_repr_enum() && field.magic.is_some() {
            Err(syn::Error::new(proc_macro2::Span::call_site(), "`repr` and `magic` are mutually exclusive"))
        } else {
            // TODO: Clone less please
            let expected_magic_kind = self.fields.get(0)
                .unwrap_or(&field)
                .magic
                .as_ref()
                .map(|field| field.0.clone());
            let magic_kind = field.magic.as_ref().map(|field| field.0.clone());

            if expected_magic_kind == magic_kind {
                self.fields.push(field);
                Ok(())
            } else if expected_magic_kind.is_some() && magic_kind.is_some() {
                // TODO: Should error on the magic token, or at least the field
                // ident
                Err(syn::Error::new(proc_macro2::Span::call_site(), format!("conflicting magic type; expected {}", expected_magic_kind.unwrap())))
            } else {
                Err(syn::Error::new(proc_macro2::Span::call_site(), "either all variants, or no variants, must have magic on a unit enum"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_struct {
        ($name:ident, $str:literal) => {
            #[test]
            fn $name() {
                let tokens: TokenStream = ($str).parse().unwrap();
                let _: StructAttr = syn::parse2(tokens).unwrap();
            }
        }
    }

    macro_rules! test_enum {
        ($name:ident, $str:literal) => {
            #[test]
            fn $name() {
                let tokens: TokenStream = ($str).parse().unwrap();
                let _: EnumAttr = syn::parse2(tokens).unwrap();
            }
        }
    }

    macro_rules! test_unit_enum {
        ($name:ident, $str:literal) => {
            #[test]
            fn $name() {
                let tokens: TokenStream = ($str).parse().unwrap();
                let _: UnitEnumAttr = syn::parse2(tokens).unwrap();
            }
        }
    }

    test_struct!(parse_struct_big, "big");
    test_struct!(parse_struct_magic, "magic = 3u8");
    test_struct!(parse_struct_magic_paren, "magic(2u16)");
    test_struct!(parse_struct_import, "import(x: u32, y: &[f32])");
    test_struct!(parse_struct_import_tuple, "import_tuple(args: (u32))");
    test_enum!(parse_enum_error, "return_all_errors");
    test_unit_enum!(parse_unit_enum_repr, "repr = u8");
    test_unit_enum!(parse_unit_enum_repr_paren, "repr(i32)");
}
