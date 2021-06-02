use syn::{
    parse::{Parse, ParseStream},
    Expr, Token,
};

pub(crate) enum NamedArgAttr {
    Default(Expr),
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
            let _: kw::try_optional = input.parse()?;
            Ok(NamedArgAttr::TryOptional)
        } else if lookahead.peek(kw::default) {
            let _: kw::default = input.parse()?;
            let _: Token![=] = input.parse()?;
            let expr: Expr = input.parse()?;

            Ok(NamedArgAttr::Default(expr))
        } else {
            Err(lookahead.error())
        }
    }
}
