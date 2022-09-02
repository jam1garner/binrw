mod attrs;
mod field_level_attrs;
mod keywords;
mod macros;
mod meta_types;
mod result;
mod top_level_attrs;
mod try_set;
mod types;

use crate::{combine_error, is_binread_attr, is_binwrite_attr, Options};
pub(crate) use field_level_attrs::{EnumVariant, StructField, UnitEnumField};
use macros::attr_struct;
use meta_types::MetaAttrList;
// TODO: Should export a processed type, not a meta type
pub(crate) use meta_types::IdentTypeMaybeDefault;
use proc_macro2::Span;
pub(crate) use result::ParseResult;
use syn::token::Token;
pub(crate) use top_level_attrs::{Enum, Input, Struct, UnitOnlyEnum};
use try_set::TrySet;
pub(crate) use types::*;

trait FromAttrs<Attr: syn::parse::Parse> {
    fn try_from_attrs(attrs: &[syn::Attribute], options: Options) -> ParseResult<Self>
    where
        Self: Default + Sized,
    {
        Self::set_from_attrs(Self::default(), attrs, options)
    }

    fn set_from_attrs(mut self, attrs: &[syn::Attribute], options: Options) -> ParseResult<Self>
    where
        Self: Sized,
    {
        let attrs = attrs
            .iter()
            .filter(|attr| {
                if options.write {
                    is_binwrite_attr(attr)
                } else {
                    is_binread_attr(attr)
                }
            })
            .flat_map(
                |attr| match syn::parse2::<MetaAttrList<Attr>>(attr.tokens.clone()) {
                    Ok(list) => either::Left(list.into_iter().map(Ok)),
                    Err(err) => either::Right(core::iter::once(Err(err))),
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

        if let Some(error) = all_errors {
            ParseResult::Partial(self, error)
        } else {
            ParseResult::Ok(self)
        }
    }

    fn try_set_attr(&mut self, attr: Attr) -> syn::Result<()>;
}

trait FromField {
    type In;

    fn from_field(field: &Self::In, index: usize, options: Options) -> ParseResult<Self>
    where
        Self: Sized;
}

trait FromInput<Attr: syn::parse::Parse>: FromAttrs<Attr> {
    type Field: FromField + 'static;

    fn from_input<'input>(
        attrs: &'input [syn::Attribute],
        fields: impl Iterator<Item = &'input <Self::Field as FromField>::In>,
        options: Options,
    ) -> ParseResult<Self>
    where
        Self: Sized + Default,
    {
        let (mut this, mut all_errors) = Self::try_from_attrs(attrs, options).unwrap_tuple();

        this.set_options(options);

        for (index, field) in fields.enumerate() {
            let (field, mut field_error) =
                Self::Field::from_field(field, index, options).unwrap_tuple();
            if field_error.is_none() {
                field_error = this.push_field(field).err();
            }

            if let Some(field_error) = field_error {
                combine_error(&mut all_errors, field_error);
            }
        }

        if let Err(validation_error) = this.validate(options) {
            combine_error(&mut all_errors, validation_error);
        }

        if let Some(error) = all_errors {
            ParseResult::Partial(this, error)
        } else {
            ParseResult::Ok(this)
        }
    }

    fn push_field(&mut self, field: Self::Field) -> syn::Result<()>;

    fn set_options(&mut self, _: Options) {}

    fn validate(&self, _: Options) -> syn::Result<()>;
}

trait KeywordToken {
    type Token: Token;

    fn display() -> &'static str {
        <Self::Token as Token>::display()
    }

    fn dyn_display(&self) -> &'static str {
        Self::display()
    }

    fn keyword_span(&self) -> Span;
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use syn::DeriveInput;

    #[cfg_attr(coverage_nightly, no_coverage)]
    fn try_input(input: TokenStream) -> ParseResult<Input> {
        Input::from_input(
            &syn::parse2::<DeriveInput>(input).unwrap(),
            Options {
                derive: false,
                write: false,
            },
        )
    }

    macro_rules! try_error (
        ($name:ident: $message:literal $tt:tt) => {
            #[test]
            #[cfg_attr(coverage_nightly, no_coverage)]
            #[should_panic(expected = $message)]
            fn $name() {
                try_input(quote::quote! $tt).unwrap();
            }
        };
        ($name:ident $tt:tt) => {
            #[test]
            #[cfg_attr(coverage_nightly, no_coverage)]
            #[should_panic]
            fn $name() {
                try_input(quote::quote! $tt).unwrap();
            }
        };
    );

    try_error!(args_calc_conflict: "`args` is incompatible" {
        struct Foo {
            #[br(args(()), calc(None))]
            a: Option<u8>,
        }
    });

    try_error!(conflicting_keyword_bool: "conflicting `restore_position` keyword" {
        struct Foo {
            #[br(restore_position, restore_position)]
            a: i32,
        }
    });

    try_error!(conflicting_keyword_count_args_list: "did you mean `args { inner: (a,) }`" {
        struct Foo {
            a: u8,
            b: u8,
            #[br(count = b, args(a))]
            c: Vec<Item>,
        }
    });

    try_error!(conflicting_keyword_count_args_list_long: "did you mean `args { inner: (a, ...) }`" {
        struct Foo {
            a: u8,
            b: u8,
            #[br(count = b, args(a, b))]
            c: Vec<Item>,
        }
    });

    try_error!(conflicting_keyword_count_args_raw: "did you mean `args { inner: a }`" {
        struct Foo {
            a: u8,
            b: u8,
            #[br(count = b, args_raw = a)]
            c: Vec<Item>,
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
        #[br(import{a: i32}, import_raw(args: (i32, )))]
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
            #[br(args { a: 3 }, args_raw = (a, ))]
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

    try_error!(err_context_missing: "requires a value" {
        struct Foo {
            #[br(err_context())]
            a: u8,
        }
    });

    try_error!(err_context_missing_format: "format string expected" {
        struct Foo {
            #[br(err_context(a, b))]
            a: u8,
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

    try_error!(invalid_magic_float: "expected explicit type suffix for float" {
        #[br(magic = 0.0)]
        struct Foo;
    });

    try_error!(invalid_magic_int: "expected explicit type suffix for integer" {
        #[br(magic = 0)]
        struct Foo;
    });

    try_error!(invalid_magic_type: "expected byte string, byte, float, or int" {
        #[br(magic = "invalid_type")]
        struct Foo;
    });

    try_error!(try_calc_conflict: "`try` is incompatible" {
        struct Foo {
            #[br(try, calc(None))]
            a: Option<u8>,
        }
    });

    try_error!(try_default_conflict: "`try` is incompatible" {
        struct Foo {
            #[br(try, default)]
            a: Option<u8>,
        }
    });

    // Errors on one field should not prevent the parser from surfacing errors
    // on other fields
    #[test]
    #[cfg_attr(coverage_nightly, no_coverage)]
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
