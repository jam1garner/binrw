use quote::ToTokens;
use super::*;
use super::super::PassedValues;

/// MetaExpr represents a key/expr pair
/// Takes two forms:
/// * ident(expr)
/// * ident = expr
/// both are always allowed
pub type MetaExpr<Keyword> = MetaValue<Keyword, Expr>;

/// MetaType represents a key/ty pair
/// Takes two forms:
/// * ident(ty)
/// * ident = ty
/// both are always allowed
pub type MetaType<Keyword> = MetaValue<Keyword, Type>;

#[derive(Debug, Clone)]
pub struct MetaValue<Keyword: Parse, Value: Parse + ToTokens> {
    pub ident: Keyword,
    pub value: Value,
}

impl<Keyword: Parse, Value: Parse + ToTokens> Parse for MetaValue<Keyword, Value> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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

impl<Keyword: Parse> ToTokens for MetaLit<Keyword> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.lit.to_tokens(tokens);
    }
}

impl<Keyword: Parse, ItemType: Parse> ToTokens for MetaList<Keyword, ItemType> {
    fn to_tokens(&self, _tokens: &mut proc_macro2::TokenStream) {}
}

impl<Keyword: Parse, Value: Parse + ToTokens> ToTokens for MetaValue<Keyword, Value> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.value.to_tokens(tokens);
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

impl<Keyword: Parse, Value: Parse + ToTokens> MetaValue<Keyword, Value> {
    pub fn get(&self) -> proc_macro2::TokenStream {
        (&self.value).into_token_stream()
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
