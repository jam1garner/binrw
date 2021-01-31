use super::*;
use super::super::PassedValues;

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
        let ident = input.parse()?;
        let content;
        let parens = parenthesized!(content in input);
        Ok(MetaList {
            ident,
            parens,
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

use quote::ToTokens;

impl<Keyword: Parse> ToTokens for MetaLit<Keyword> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.lit.to_tokens(tokens);
    }
}

impl<Keyword: Parse, ItemType: Parse> ToTokens for MetaList<Keyword, ItemType> {
    fn to_tokens(&self, _tokens: &mut proc_macro2::TokenStream) {}
}

impl<Keyword: Parse> ToTokens for MetaExpr<Keyword> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.expr.to_tokens(tokens);
    }
}

impl<Keyword: Parse> ToTokens for MetaFunc<Keyword> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.func {
            MetaFuncExpr::Path(p) => p.to_tokens(tokens),
            MetaFuncExpr::Closure(c) => c.to_tokens(tokens)
        }
    }
}

impl<Keyword: Parse> MetaExpr<Keyword> {
    pub fn get(&self) -> proc_macro2::TokenStream {
        (&self.expr).into_token_stream()
    }
}

impl<Keyword: Parse> MetaFunc<Keyword> {
    pub fn get(&self) -> proc_macro2::TokenStream {
        self.into_token_stream()
    }
}

impl<Keyword: Parse> MetaLit<Keyword> {
    pub fn get(&self) -> Lit {
        self.lit.clone()
    }
}

impl<Keyword: Parse> MetaList<Keyword, Expr> {
    pub fn get(&self) -> PassedValues {
        PassedValues(self.fields.iter().map(ToTokens::into_token_stream).collect())
    }
}
