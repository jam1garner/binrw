#[macro_use]
pub(crate) mod macros;
mod attrs;
mod field_level_attrs;
mod keywords;
pub(crate) mod meta_types;
mod top_level_attrs;
mod types;

pub(crate) use field_level_attrs::*;
use meta_types::MetaAttrList;
use proc_macro2::Span;
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

pub(crate) fn is_binread_attr(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("br") || attr.path.is_ident("binread")
}

pub(crate) trait FromAttrs<Attr: syn::parse::Parse> {
    fn try_from_attrs(attrs: &[syn::Attribute]) -> ParseResult<Self>
    where
        Self: Default + Sized,
    {
        Self::set_from_attrs(Self::default(), attrs)
    }

    fn set_from_attrs(mut self, attrs: &[syn::Attribute]) -> ParseResult<Self>
    where
        Self: Sized,
    {
        #[allow(clippy::filter_map)]
        let attrs = attrs
            .iter()
            .filter(|attr| is_binread_attr(attr))
            .flat_map(
                |attr| match syn::parse2::<MetaAttrList<Attr>>(attr.tokens.clone()) {
                    Ok(list) => either::Either::Right(list.into_iter().map(Ok)),
                    Err(err) => either::Either::Left(core::iter::once(Err(err))),
                },
            );

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

        // https://github.com/rust-lang/rust-clippy/issues/5822
        #[allow(clippy::option_if_let_else)]
        if let Some(error) = all_errors {
            ParseResult::Partial(self, error)
        } else {
            ParseResult::Ok(self)
        }
    }

    fn try_set_attr(&mut self, attr: Attr) -> syn::Result<()>;
}

pub(crate) trait FromField {
    type In;

    fn from_field(field: &Self::In, index: usize) -> ParseResult<Self>
    where
        Self: Sized;
}

pub(crate) trait FromInput<Attr: syn::parse::Parse>: FromAttrs<Attr> {
    type Field: FromField + 'static;

    fn from_input<'input>(
        attrs: &'input [syn::Attribute],
        fields: impl Iterator<Item = &'input <Self::Field as FromField>::In>,
    ) -> ParseResult<Self>
    where
        Self: Sized + Default,
    {
        let (mut this, mut all_errors) = Self::try_from_attrs(attrs).unwrap_tuple();

        for (index, field) in fields.enumerate() {
            let (field, mut field_error) = Self::Field::from_field(field, index).unwrap_tuple();
            if field_error.is_none() {
                field_error = this.push_field(field).err();
            }

            if let Some(field_error) = field_error {
                combine_error(&mut all_errors, field_error);
            }
        }

        if let Err(validation_error) = this.validate() {
            combine_error(&mut all_errors, validation_error);
        }

        // https://github.com/rust-lang/rust-clippy/issues/5822
        #[allow(clippy::option_if_let_else)]
        if let Some(error) = all_errors {
            ParseResult::Partial(this, error)
        } else {
            ParseResult::Ok(this)
        }
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

impl<T: Token + Spanned> KeywordToken for T {
    type Token = T;

    fn keyword_span(&self) -> Span {
        self.span()
    }
}

pub(crate) enum PartialResult<T, E> {
    Ok(T),
    Partial(T, E),
    Err(E),
}

impl<T, E> PartialResult<T, E> {
    #[cfg(test)]
    pub(crate) fn err(self) -> Option<E> {
        match self {
            PartialResult::Ok(_) => None,
            PartialResult::Partial(_, error) | PartialResult::Err(error) => Some(error),
        }
    }

    pub(crate) fn map<F, U>(self, f: F) -> PartialResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            PartialResult::Ok(value) => PartialResult::Ok(f(value)),
            PartialResult::Partial(value, error) => PartialResult::Partial(f(value), error),
            PartialResult::Err(error) => PartialResult::Err(error),
        }
    }

    pub(crate) fn ok(self) -> Option<T> {
        match self {
            PartialResult::Ok(value) | PartialResult::Partial(value, _) => Some(value),
            PartialResult::Err(_) => None,
        }
    }
}

impl<T, E: core::fmt::Debug> PartialResult<T, E> {
    #[cfg(test)]
    pub(crate) fn unwrap(self) -> T {
        match self {
            PartialResult::Ok(value) => value,
            PartialResult::Partial(_, error) => panic!(
                "called `PartialResult::unwrap() on a `Partial` value: {:?}",
                &error
            ),
            PartialResult::Err(error) => panic!(
                "called `PartialResult::unwrap() on an `Err` value: {:?}",
                &error
            ),
        }
    }

    pub(crate) fn unwrap_tuple(self) -> (T, Option<E>) {
        match self {
            PartialResult::Ok(value) => (value, None),
            PartialResult::Partial(value, error) => (value, Some(error)),
            PartialResult::Err(error) => panic!(
                "called `PartialResult::unwrap_tuple() on an `Err` value: {:?}",
                &error
            ),
        }
    }
}

pub(crate) type ParseResult<T> = PartialResult<T, syn::Error>;

pub(crate) trait TrySet<T> {
    fn try_set(self, to: &mut T) -> syn::Result<()>;
}

impl<T: KeywordToken> TrySet<bool> for T {
    fn try_set(self, to: &mut bool) -> syn::Result<()> {
        if *to {
            Err(syn::Error::new(
                self.keyword_span(),
                format!("conflicting {} keyword", self.dyn_display()),
            ))
        } else {
            *to = true;
            Ok(())
        }
    }
}

// TODO: This sucks
pub(crate) enum TrySetError {
    Infallible,
    Syn(syn::Error),
}

impl From<core::convert::Infallible> for TrySetError {
    fn from(_: core::convert::Infallible) -> Self {
        Self::Infallible
    }
}

impl From<syn::Error> for TrySetError {
    fn from(error: syn::Error) -> Self {
        Self::Syn(error)
    }
}

impl<T: core::convert::TryInto<To, Error = E> + KeywordToken, E: Into<TrySetError>, To>
    TrySet<Option<To>> for T
{
    fn try_set(self, to: &mut Option<To>) -> syn::Result<()> {
        if to.is_none() {
            *to = Some(self.try_into().map_err(|error| match error.into() {
                TrySetError::Infallible => unreachable!(),
                TrySetError::Syn(error) => error,
            })?);
            Ok(())
        } else {
            Err(syn::Error::new(
                self.keyword_span(),
                format!("conflicting {} keyword", self.dyn_display()),
            ))
        }
    }
}

impl<T: core::convert::TryInto<To, Error = syn::Error> + KeywordToken, To> TrySet<Vec<To>> for T {
    fn try_set(self, to: &mut Vec<To>) -> syn::Result<()> {
        to.push(self.try_into()?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::DeriveInput;

    fn try_input(input: TokenStream) -> ParseResult<Input> {
        Input::from_input(&syn::parse2::<DeriveInput>(input).unwrap())
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

    try_error!(conflicting_keyword_bool: "conflicting `restore_position` keyword" {
        struct Foo {
            #[br(restore_position, restore_position)]
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

    try_error!(conflicting_keyword_read_mode: "conflicting read mode keyword" {
        struct Foo {
            #[br(calc(1), default, ignore, parse_with = u8)]
            a: i32,
        }
    });

    try_error!(enum_missing_magic_repr {
        enum UnitEnum {
            A,
        }
    });

    try_error!(invalid_assert_args: "too many arguments" {
        #[br(assert(false, String::from("message"), "too", "many", "arguments"))]
        struct Foo;
    });

    try_error!(invalid_assert_empty: "requires a boolean expression" {
        #[br(assert())]
        struct Foo;
    });

    try_error!(invalid_if_args: "too many arguments" {
        struct Foo {
            #[br(if(false, 0, 1, 2, 3))]
            a: u8,
        }
    });

    try_error!(invalid_if_empty: "requires a boolean expression" {
        struct Foo {
            #[br(if())]
            a: u8,
        }
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

    try_error!(invalid_magic_type: "expected byte string, byte, char, float, or int" {
        #[br(magic = "invalid_type")]
        struct Foo;
    });

    try_error!(magic_conflict: "conflicting magic types" {
        enum Foo {
            #[br(magic = 0u8)] A,
            #[br(magic = 1i16)] B,
        }
    });

    // Errors on one field should not prevent the parser from surfacing errors
    // on other fields
    #[test]
    fn non_blocking_errors() {
        let error = try_input(quote::quote! {
            #[br(invalid_keyword_struct)]
            struct Foo {
                #[br(invalid_keyword_struct_field_a)]
                a: i32,
                #[br(invalid_keyword_struct_field_b)]
                b: i32,
            }
        })
        .err()
        .unwrap();
        assert_eq!(error.into_iter().count(), 3);
    }

    try_error!(repr_magic_conflict: "mutually exclusive" {
        #[br(repr = u8)]
        enum Foo {
            #[br(magic = 0u8)] A,
        }
    });

    try_error!(deref_now_offset_after_conflict: "mutually exclusive" {
        struct Foo {
            #[br(deref_now, offset_after(1))]
            a: u8,
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
