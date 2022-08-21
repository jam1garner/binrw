use syn::{
    parse::{Parse, ParseStream},
    Expr, Token,
};

pub(crate) enum NamedArgAttr {
    Default(Box<Expr>),
    TryOptional,
}

mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(try_optional);
}

impl Parse for NamedArgAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::try_optional) {
            input.parse::<kw::try_optional>()?;
            Ok(NamedArgAttr::TryOptional)
        } else if lookahead.peek(kw::default) {
            input.parse::<kw::default>()?;
            input.parse::<Token![=]>()?;
            Ok(NamedArgAttr::Default(Box::new(input.parse()?)))
        } else {
            Err(lookahead.error())
        }
    }
}
