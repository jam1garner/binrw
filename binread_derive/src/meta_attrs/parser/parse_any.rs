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
        pub enum $enum {
            $variant1($ty1),
            $(
                $variantn($tyn)
            ),*
        }

        impl syn::parse::Parse for $enum {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let x = input.parse().map(Self::$variant1);
                $(
                    let x = x.or_else(|_: syn::Error|{
                            Ok(Self::$variantn(input.parse()?))
                        });
                )*
                x.map_err(|_: syn::Error|{
                    input.error(concat!(
                        "Cannot parse, expected one of the following: ",
                        stringify!($variant1)
                        $(
                            ,", ",
                            stringify!($variantn)
                        )*
                    ))
                })
            }
        }
    };
}
