use super::*;

/// MetaExpr represents a key/expr pair
/// Takes two forms:
/// * ident(expr)
/// * ident = expr
/// both are always allowed
#[derive(Debug, Clone)]
pub struct MetaExpr<Keyword: Parse> {
    pub ident: Keyword,
    pub expr: Expr,
}

impl<Keyword: Parse> Parse for MetaExpr<Keyword> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let expr = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            content.parse()?
        } else {
            input.parse::<Token![=]>()?;
            input.parse()?
        };

        Ok(MetaExpr {
            ident,
            expr
        })
    }
}

type EqToken = Token![=];

#[derive(Debug, Clone)]
pub struct MetaList<Keyword: Parse, ItemType: Parse> {
    pub ident: Keyword,
    pub parens: token::Paren,
    pub fields: Vec<ItemType>,
}

impl<Keyword: Parse, ItemType: Parse> Parse for MetaList<Keyword, ItemType> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(MetaList {
            ident: input.parse()?,
            parens: parenthesized!(content in input),
            fields: content.parse_terminated::<_, Token![,]>(ItemType::parse)?.into_iter().collect()
        })
    }
}

#[derive(Debug, Clone)]
pub struct MetaFunc<Keyword: Parse> {
    pub ident: Keyword,
    pub eq: EqToken,
    pub func: MetaFuncExpr,
}

impl<Keyword: Parse> Parse for MetaFunc<Keyword> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(MetaFunc {
            ident: input.parse()?,
            eq: input.parse()?,
            func: input.parse()?
        })
    }
}

#[derive(Debug, Clone)]
pub struct MetaLit<Keyword: Parse> {
    pub ident: Keyword,
    pub lit: Lit,
}

impl<Keyword: Parse> Parse for MetaLit<Keyword> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let lit = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            content.parse()?
        } else {
            input.parse::<Token![=]>()?;
            input.parse()?
        };

        Ok(MetaLit {
            ident,
            lit
        })
    }
}
