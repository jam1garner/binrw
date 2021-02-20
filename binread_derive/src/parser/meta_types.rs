use proc_macro2::TokenStream;
use quote::ToTokens;
use super::{KeywordToken, keywords as kw};
use syn::{Expr, Lit, Token, Type, parenthesized, parse::{Parse, ParseStream}, punctuated::Punctuated, token};

type Fields<T> = Punctuated<T, Token![,]>;

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

impl <Keyword, Value: ToTokens> From<MetaValue<Keyword, Value>> for TokenStream {
    fn from(value: MetaValue<Keyword, Value>) -> Self {
        value.value.into_token_stream()
    }
}

impl <Keyword, Value: ToTokens> ToTokens for MetaValue<Keyword, Value> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.value.to_tokens(tokens);
    }
}

impl <Keyword: syn::token::Token + KeywordToken, Value> KeywordToken for MetaValue<Keyword, Value> {
    type Token = Keyword;

    fn keyword_span(&self) -> proc_macro2::Span {
        self.ident.keyword_span()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MetaList<Keyword, ItemType> {
    pub ident: Keyword,
    pub parens: token::Paren,
    pub fields: Fields<ItemType>,
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

impl <Keyword: syn::token::Token + KeywordToken, ItemType> KeywordToken for MetaList<Keyword, ItemType> {
    type Token = Keyword;

    fn keyword_span(&self) -> proc_macro2::Span {
        self.ident.keyword_span()
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
    type Token = kw::import_tuple;

    fn keyword_span(&self) -> proc_macro2::Span {
        self.ident.keyword_span()
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse_ty {
        ($name:ident, $str:literal, $ty:ty) => {
            #[test]
            fn $name() {
                let tokens: TokenStream = ($str).parse().unwrap();
                let _: $ty = syn::parse2(tokens).unwrap();
            }
        }
    }

    macro_rules! parse_ty_fail {
        ($name:ident, $str:literal, $ty:ty) => {
            #[test]
            #[should_panic]
            fn $name() {
                let tokens: TokenStream = ($str).parse().unwrap();
                let _: $ty = syn::parse2(tokens).unwrap();
            }
        }
    }

    parse_ty!(meta_bool, "little", kw::little);
    parse_ty!(meta_lit, "magic = 3u8", MetaLit<kw::magic>);
    parse_ty!(meta_byte_lit, "magic = b\"TEST\"", MetaLit<kw::magic>);
    parse_ty!(meta_str_lit, "magic = \"string\"", MetaLit<kw::magic>);
    parse_ty!(meta_func_closure, "map = |x| x + 1", MetaExpr<kw::map>);
    parse_ty!(meta_func_path, "map = ToString::to_string", MetaExpr<kw::map>);
    parse_ty!(meta_func_fn_expr, "map = {|| { |x| x + 1 }()}", MetaExpr<kw::map>);
    parse_ty!(meta_ty, "repr = u8", MetaType<kw::repr>);

    parse_ty_fail!(meta_lit_panic, "= 3u8", MetaLit<kw::magic>);
    parse_ty_fail!(meta_lit_panic2, "test = 3u8", MetaLit<kw::magic>);
    parse_ty_fail!(meta_ty_panic, "repr = 3u8", MetaType<kw::repr>);
}
