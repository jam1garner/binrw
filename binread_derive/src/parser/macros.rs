/// Attempt to parse variants in order until a match is found
macro_rules! parse_any {
    (enum $enum:ident {
        $(
            $variant:ident($ty:ty)
        ),*
        $(,)?
    }) => {
        #[derive(Debug, Clone)]
        pub(crate) enum $enum {
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
