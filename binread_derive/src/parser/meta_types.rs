use quote::ToTokens;
use super::{KeywordToken, keywords as kw};
use syn::{Expr, Lit, Token, Type, parenthesized, parse::{Parse, ParseStream}, punctuated::Punctuated, token};

/// `MetaExpr` represents a key/expr pair
/// Takes two forms:
/// * ident(expr)
/// * ident = expr
/// both are always allowed
pub(crate) type MetaExpr<Keyword> = MetaValue<Keyword, Expr>;

/// `MetaType` represents a key/ty pair
/// Takes two forms:
/// * ident(ty)
/// * ident = ty
/// both are always allowed
pub(crate) type MetaType<Keyword> = MetaValue<Keyword, Type>;

/// `MetaLit` represents a key/lit pair
/// Takes two forms:
/// * ident(lit)
/// * ident = lit
/// both are always allowed
pub(crate) type MetaLit<Keyword> = MetaValue<Keyword, Lit>;

/// `MetaFunc` represents a key/fn pair
/// Takes two forms:
/// * ident(fn)
/// * ident = fn
/// both are always allowed
pub(crate) type MetaFunc<Keyword> = MetaValue<Keyword, MetaFuncExpr>;

#[derive(Debug, Clone)]
pub(crate) struct MetaValue<Keyword, Value> {
    pub ident: Keyword,
    pub value: Value,
}

impl <Keyword: Parse, Value: Parse> Parse for MetaValue<Keyword, Value> {
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

impl <Keyword, Value: ToTokens> ToTokens for MetaValue<Keyword, Value> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.value.to_tokens(tokens);
    }
}

impl <Keyword: KeywordToken, Value> KeywordToken for MetaValue<Keyword, Value> {
    fn display() -> &'static str {
        <Keyword as KeywordToken>::display()
    }
}

type Fields<T> = Punctuated<T, Token![,]>;

#[derive(Debug, Clone)]
pub(crate) struct MetaList<Keyword, ItemType> {
    pub ident: Keyword,
    pub parens: token::Paren,
    pub fields: Fields<ItemType>,
}

impl <Keyword, ItemType: ToTokens> MetaList<Keyword, ItemType> {
    pub fn get(&self) -> Vec<proc_macro2::TokenStream> {
        self.fields.iter().map(ToTokens::into_token_stream).collect()
    }
}

impl <Keyword: Parse, ItemType: Parse> Parse for MetaList<Keyword, ItemType> {
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

impl <Keyword: KeywordToken, ItemType> KeywordToken for MetaList<Keyword, ItemType> {
    fn display() -> &'static str {
        <Keyword as KeywordToken>::display()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MetaFuncExpr {
    Path(syn::Path),
    Closure(syn::ExprClosure)
}

impl Parse for MetaFuncExpr {
    #[allow(clippy::map_err_ignore)]
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        input.parse()
            .map(Self::Path)
            .or_else(|_: syn::Error| Ok(Self::Closure(input.parse()?)))
            .map_err(|_: syn::Error| input.error("expected path or closure"))
    }
}

impl ToTokens for MetaFuncExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Path(p) => p.to_tokens(tokens),
            Self::Closure(c) => c.to_tokens(tokens),
        }
    }
}

// This is like `syn::PatType` except:
// (1) Implements `Parse`;
// (2) No attributes;
// (3) Only allows an ident on the LHS instead of any `syn::Pat`.
#[derive(Debug, Clone)]
pub(crate) struct IdentPatType {
    pub ident: syn::Ident,
    pub colon_token: Token![:],
    pub ty: syn::Type
}

impl Parse for IdentPatType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(IdentPatType {
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ImportArgTuple {
    pub ident: kw::import_tuple,
    pub parens: token::Paren,
    pub arg: IdentPatType
}

impl Parse for ImportArgTuple {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let content;
        let parens = parenthesized!(content in input);
        Ok(ImportArgTuple {
            ident,
            parens,
            arg: content.parse()?
        })
    }
}

impl KeywordToken for ImportArgTuple {
    fn display() -> &'static str {
        <kw::import_tuple as KeywordToken>::display()
    }
}

pub(crate) struct MetaAttrList<P>(Fields<P>);

impl <P> MetaAttrList<P> {
    pub(crate) fn into_iter(self) -> impl Iterator<Item = P> {
        self.0.into_iter()
    }
}

impl <P: Parse> Parse for MetaAttrList<P> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(MetaAttrList(
            Fields::parse_terminated(&content)?
        ))
    }
}
