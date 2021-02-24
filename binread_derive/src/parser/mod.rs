#[macro_use]
pub(crate) mod macros;
mod attrs;
mod field_level_attrs;
mod keywords;
pub(crate) mod meta_types;
mod top_level_attrs;
mod types;

pub(crate) use field_level_attrs::*;
use proc_macro2::Span;
use meta_types::MetaAttrList;
use syn::{spanned::Spanned, token::Token};
pub(crate) use top_level_attrs::*;
pub(crate) use types::*;

fn combine_error(all_errors: &mut Option<syn::Error>, new_error: syn::Error) {
    if let Some(all_errors) = all_errors {
        all_errors.combine(new_error);
    } else {
        *all_errors = Some(new_error);
    }
}

pub(crate) trait FromAttrs<Attr: syn::parse::Parse> {
    fn try_from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> where Self: Default + Sized {
        Self::set_from_attrs(Self::default(), attrs)
    }

    fn set_from_attrs(mut self, attrs: &[syn::Attribute]) -> syn::Result<Self> where Self: Sized {
        #[allow(clippy::filter_map)]
        let attrs = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("br") || attr.path.is_ident("binread"))
            .flat_map(|attr| {
                match syn::parse2::<MetaAttrList<Attr>>(attr.tokens.clone()) {
                    Ok(list) => either::Either::Right(list.into_iter().map(Ok)),
                    Err(err) => either::Either::Left(core::iter::once(Err(err))),
                }
            });

        let mut all_errors = None::<syn::Error>;
        for attr in attrs {
            let result = match attr {
                Ok(attr) => self.try_set_attr(attr),
                Err(e) => Err(e),
            };

            if let Err(parse_error) = result {
                combine_error(&mut all_errors, parse_error);
            }
        }

        match all_errors {
            Some(error) => Err(error),
            None => Ok(self),
        }
    }

    fn try_set_attr(&mut self, attr: Attr) -> syn::Result<()>;
}

pub(crate) trait FromField {
    type In;

    fn from_field(field: &Self::In) -> syn::Result<Self> where Self: Sized;
}

pub(crate) trait FromInput<Attr: syn::parse::Parse>: FromAttrs<Attr> {
    type Field: FromField + 'static;

    fn from_input<'input>(attrs: &'input [syn::Attribute], fields: impl Iterator<Item = &'input <Self::Field as FromField>::In>) -> syn::Result<Self> where Self: Sized + Default {
        // TODO: This probably should return an incomplete object + error so
        // that all field validation can occur; currently if the parent object
        // has an error then any validation in `push_field` or `validate` will
        // not occur.
        let (mut this, mut all_errors) = match Self::try_from_attrs(attrs) {
            Ok(this) => (Some(this), None),
            Err(all_errors) => (None, Some(all_errors)),
        };

        for field in fields {
            let field_error = match Self::Field::from_field(field) {
                Ok(field) => match &mut this {
                    Some(this) => this.push_field(field),
                    None => Ok(()),
                },
                Err(field_error) => Err(field_error),
            };

            if let Err(field_error) = field_error {
                combine_error(&mut all_errors, field_error);
            }
        }

        if let Some(ref this) = this {
            if let Err(validation_error) = this.validate() {
                combine_error(&mut all_errors, validation_error);
            }
        }

        all_errors.map_or_else(|| Ok(this.unwrap()), Err)
    }

    fn push_field(&mut self, field: Self::Field) -> syn::Result<()>;

    fn validate(&self) -> syn::Result<()> {
        Ok(())
    }
}

pub(crate) trait KeywordToken {
    type Token: Token;

    fn display() -> &'static str {
        <Self::Token as Token>::display()
    }

    fn dyn_display(&self) -> &'static str {
        Self::display()
    }

    fn keyword_span(&self) -> Span;
}

impl <T: Token + Spanned> KeywordToken for T {
    type Token = T;

    fn keyword_span(&self) -> Span {
        self.span()
    }
}

pub(crate) trait TrySet<T> {
    fn try_set(self, to: &mut T) -> syn::Result<()>;
}

impl <T: KeywordToken> TrySet<bool> for T {
    fn try_set(self, to: &mut bool) -> syn::Result<()> {
        if *to {
            Err(syn::Error::new(self.keyword_span(), format!("conflicting {} keyword", self.dyn_display())))
        } else {
            *to = true;
            Ok(())
        }
    }
}

impl <T: Into<To> + KeywordToken, To> TrySet<Option<To>> for T {
    fn try_set(self, to: &mut Option<To>) -> syn::Result<()> {
        if to.is_none() {
            *to = Some(self.into());
            Ok(())
        } else {
            Err(syn::Error::new(self.keyword_span(), format!("conflicting {} keyword", self.dyn_display())))
        }
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use super::*;
    use syn::DeriveInput;

    fn try_input(input: TokenStream) -> syn::Result<Input> {
        Input::from_input(&syn::parse2::<DeriveInput>(input)?)
    }

    macro_rules! try_error (
        ($name:ident: $message:literal $tt:tt) => {
            #[test]
            #[should_panic(expected = $message)]
            fn $name() {
                try_input(quote::quote! $tt).unwrap();
            }
        };
        ($name:ident $tt:tt) => {
            #[test]
            #[should_panic]
            fn $name() {
                try_input(quote::quote! $tt).unwrap();
            }
        };
    );

    try_error!(conflicting_keyword_bool: "conflicting `ignore` keyword" {
        struct Foo {
            #[br(ignore, ignore)]
            a: i32,
        }
    });

    try_error!(conflicting_keyword_cond_endian: "conflicting endianness keyword" {
        struct Foo {
            #[br(big, little, is_big = true, is_little = true)]
            a: i32,
        }
    });

    try_error!(conflicting_keyword_enum_error_mode: "conflicting error handling keyword" {
        #[br(return_all_errors, return_unexpected_error)]
        enum Foo {
            A(i32),
        }
    });

    try_error!(conflicting_keyword_imports: "conflicting import keyword" {
        #[br(import(a: i32), import_tuple(args: (i32, )))]
        struct Foo;
    });

    try_error!(conflicting_keyword_map: "conflicting map keyword" {
        struct Foo {
            #[br(map = |_| 0, try_map = |_| Ok(0))]
            a: i32,
        }
    });

    try_error!(conflicting_keyword_option: "conflicting `magic` keyword" {
        #[br(magic = 0u8, magic = 0u8)]
        struct Foo;
    });

    try_error!(conflicting_keyword_passed_args: "conflicting args keyword" {
        struct Foo {
            a: i32,
            #[br(args(a), args_tuple = (a, ))]
            b: i32,
        }
    });

    try_error!(enum_missing_magic_repr {
        enum UnitEnum {
            A,
        }
    });

    try_error!(invalid_assert_args: "too many arguments" {
        #[br(assert(false, "message", "too", "many", "arguments"))]
        struct Foo;
    });

    try_error!(invalid_assert_empty: "requires a boolean expression" {
        #[br(assert())]
        struct Foo;
    });

    try_error!(invalid_keyword_enum_variant: "expected one of" {
        enum Enum {
            #[br(invalid_enum_variant_keyword)]
            A(i32),
        }
    });

    try_error!(invalid_keyword_enum: "expected one of" {
        #[br(invalid_enum_keyword)]
        enum Enum {
            A(i32),
        }
    });

    try_error!(invalid_keyword_struct_field: "expected one of" {
        struct Struct {
            #[br(invalid_struct_field_keyword)]
            field: i32,
        }
    });

    try_error!(invalid_keyword_struct: "expected one of" {
        #[br(invalid_struct_keyword)]
        struct Struct {
            field: i32,
        }
    });

    try_error!(invalid_keyword_unit_enum_field: "expected one of" {
        #[br(repr = u8)]
        enum UnitEnum {
            #[br(invalid_unit_enum_field_keyword)]
            A,
        }
    });

    try_error!(invalid_keyword_unit_enum: "expected one of" {
        #[br(invalid_unit_enum_keyword)]
        enum UnitEnum {
            #[br(magic = 0u8)]
            A,
        }
    });

    try_error!(magic_conflict: "conflicting magic types" {
        enum Foo {
            #[br(magic = 0u8)] A,
            #[br(magic = 1i16)] B,
        }
    });

    try_error!(repr_magic_conflict: "mutually exclusive" {
        #[br(repr = u8)]
        enum Foo {
            #[br(magic = 0u8)] A,
        }
    });

    try_error!(unsupported_type_enum: "null enums are not supported" {
        enum Foo {}
    });

    try_error!(unsupported_type_union: "unions are not supported" {
        union Bar {
            a: i32,
        }
    });
}
