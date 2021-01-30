/// Attempt to parse variants in order until a match is found
macro_rules! parse_any {
    (enum $enum:ident {
        $variant1:ident($ty1:ty),
        $(
            $variantn:ident($tyn:ty)
        ),*
        $(,)?
    }) => {
        #[derive(Debug, Clone)]
        pub(crate) enum $enum {
            $variant1($ty1),
            $(
                $variantn($tyn)
            ),*
        }

        impl syn::parse::Parse for $enum {
            fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
                let x = input.parse().map(Self::$variant1);
                $(
                    let x = x.or_else(|_: syn::Error| {
                        Ok(Self::$variantn(input.parse()?))
                    });
                )*
                x.map_err(|_: syn::Error| {
                    let mut error = format!("invalid keyword, expected one of {}", <$ty1 as $crate::parser::KeywordToken>::display());
                    $(
                        error.push_str(", ");
                        error.push_str(<$tyn as $crate::parser::KeywordToken>::display());
                    )*
                    input.error(error)
                })
            }
        }
    };
}

macro_rules! only_first {
    ($obj:ident.$field:ident, $kw:expr) => {
        if $obj.$field.is_some() {
            return $crate::parser::duplicate_attr(&$kw);
        }
    }
}
