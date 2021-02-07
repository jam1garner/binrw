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

        impl syn::parse::Parse for $enum {
            fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
                $(if <<$ty as $crate::parser::KeywordToken>::Token as syn::token::Token>::peek(input.cursor()) {
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
