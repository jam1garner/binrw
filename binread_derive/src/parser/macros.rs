/// Attempt to parse variants in order until a match is found
macro_rules! parse_any {
    (enum $enum:ident {
        $(
            $variant:ident($ty:ty)
        ),*
        $(,)?
    }) => {
        pub(super) enum $enum {
            $(
                $variant($ty)
            ),*
        }

        impl ::syn::parse::Parse for $enum {
            fn parse(input: ::syn::parse::ParseStream<'_>) -> ::syn::Result<Self> {
                $(if <<$ty as $crate::parser::KeywordToken>::Token as ::syn::token::Token>::peek(input.cursor()) {
                    input.parse().map(Self::$variant)
                } else)* {
                    let mut error = String::from("expected one of: ");
                    $(
                        error.push_str(<$ty as $crate::parser::KeywordToken>::display());
                        error.push_str(", ");
                    )*
                    error.truncate(error.len() - 2);
                    Err(input.error(error))
                }
            }
        }
    };
}

// The way this works sucks for a couple reasons which are not really worth
// dealing with right now, but maybe are worth dealing with in the future:
//
// 1. Using a separate enum just for parsing, instead of implementing parsing
// within a generated struct, shouldn’t really be necessary, but seemed to be
// the simplest to make everything work within the confines of the syn API.
// There is no way to get a `ParseStream` in syn other than to implement
// `syn::parse::Parse`, and that API return signature is `Result<Self>`, but the
// parser should to be able to return partial results instead (as it does now),
// so it’d be necessary to instead implement `Parse` for `PartialResult` and
// then go through an internal API that actually does parsing (and probably also
// reimplements other stuff like `Punctuated` since there would no longer be a
// type containing all the possible directives). It would be possible also to
// attach errors to the structs themselves, but it did not seem like the extra
// work to move non-fatal errors there was really worth the added effort since
// the current design was already written and functioning.
//
// 2. The variant-to-field mapping is awful. The `from` attributes should be
// taking types instead of idents, but can’t because then there would be no way
// to generate the enum variants. Variant names could be provided separately,
// but this would clutter the call sites for no particularly good reason—the
// types are normalised enough that it’s possible to just fill out the rest of
// the type here, even though it’s a nasty obfuscation that will confuse anyone
// that doesn’t look at what the macro is doing.
//
// So, you know… here be dragons, and I’m sorry in advance.
macro_rules! attr_struct {
    (
        #[from($attr_ty:ident)]
        $(#[$meta:meta])*
        $vis:vis struct $ident:ident {
        $(
            $(#[from($($field_attr_id:ident),+)])?
            $field_vis:vis $field:ident : $field_ty:ty
        ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $ident {
            $(
                $field_vis $field: $field_ty
            ),+
        }

        impl $crate::parser::FromAttrs<$attr_ty> for $ident {
            fn try_set_attr(&mut self, attr: $attr_ty) -> ::syn::Result<()> {
                match attr {
                    $($(
                        $($attr_ty::$field_attr_id(value) => value.try_set(&mut self.$field),)+
                    )?)+
                }
            }
        }

        parse_any! {
            enum $attr_ty {
                $($(
                    $($field_attr_id($crate::parser::attrs::$field_attr_id),)+
                )?)+
            }
        }
    }
}
