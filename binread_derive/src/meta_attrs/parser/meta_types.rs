use quote::ToTokens;
use super::MetaFuncExpr;
use syn::{Expr, Lit, Token, Type, parenthesized, parse::{Parse, ParseStream}, punctuated::Punctuated, token};

/// `MetaExpr` represents a key/expr pair
/// Takes two forms:
/// * ident(expr)
/// * ident = expr
/// both are always allowed
pub type MetaExpr<Keyword> = MetaValue<Keyword, Expr>;

/// `MetaType` represents a key/ty pair
/// Takes two forms:
/// * ident(ty)
/// * ident = ty
/// both are always allowed
pub type MetaType<Keyword> = MetaValue<Keyword, Type>;

/// `MetaLit` represents a key/lit pair
/// Takes two forms:
/// * ident(lit)
/// * ident = lit
/// both are always allowed
pub type MetaLit<Keyword> = MetaValue<Keyword, Lit>;

/// `MetaFunc` represents a key/fn pair
/// Takes two forms:
/// * ident(fn)
/// * ident = fn
/// both are always allowed
pub type MetaFunc<Keyword> = MetaValue<Keyword, MetaFuncExpr>;

#[derive(Debug, Clone)]
pub struct MetaValue<Keyword: Parse, Value: Parse + ToTokens> {
    pub ident: Keyword,
    pub value: Value,
}

impl<Keyword: Parse, Value: Parse + ToTokens> MetaValue<Keyword, Value> {
    pub fn get(&self) -> proc_macro2::TokenStream {
        self.value.to_token_stream()
    }
}

impl<Keyword: Parse, Value: Parse + ToTokens> Parse for MetaValue<Keyword, Value> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let value = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            content.parse()?
        } else {
            input.parse::<Token![=]>()?;
            input.parse()?
        };

        Ok(MetaValue {
            ident,
            value,
        })
    }
}

impl<Keyword: Parse, Value: Parse + ToTokens> ToTokens for MetaValue<Keyword, Value> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.value.to_tokens(tokens);
    }
}

type Fields<T> = Punctuated<T, Token![,]>;

#[derive(Debug, Clone)]
pub struct MetaList<Keyword: Parse, ItemType: Parse> {
    pub ident: Keyword,
    pub parens: token::Paren,
    pub fields: Fields<ItemType>,
}

impl<Keyword: Parse, ItemType: Parse> Parse for MetaList<Keyword, ItemType> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let content;
        let parens = parenthesized!(content in input);
        Ok(MetaList {
            ident,
            parens,
            fields: content.parse_terminated::<_, Token![,]>(ItemType::parse)?
        })
    }
}

impl<Keyword: Parse> MetaList<Keyword, Expr> {
    pub fn get(&self) -> Vec<proc_macro2::TokenStream> {
        self.fields.iter().map(ToTokens::into_token_stream).collect()
    }
}
