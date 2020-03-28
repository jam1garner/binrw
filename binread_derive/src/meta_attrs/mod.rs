mod parser;
mod spanned_value;
mod top_level_attrs;
mod field_level_attrs;
pub(crate) use top_level_attrs::TopLevelAttrs;
pub(crate) use field_level_attrs::FieldLevelAttrs;
pub(crate) use spanned_value::SpannedValue;

use darling::{FromField, FromDeriveInput, FromVariant};
use proc_macro2::TokenStream;
use crate::compiler_error::SpanError;
use syn::{Ident, Lit, NestedMeta, Path, Type, Field, Meta, Expr, spanned::Spanned};
use quote::ToTokens;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Assert(pub TokenStream, pub Option<TokenStream>);

#[derive(Debug)]
struct MultiformExpr(TokenStream);

pub struct ParseField(Field);

#[derive(Debug, Default, Clone)]
pub struct PassedValues(Vec<TokenStream>);

impl PassedValues {
    pub fn iter(&self) -> impl Iterator<Item = &TokenStream> {
        self.0.iter()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Imports(pub Vec<Ident>, pub Vec<Type>);

impl Imports {
    pub fn idents<'a>(&'a self) -> impl Iterator<Item=&'a Ident> {
        self.0.iter()
    }
    
    pub fn types<'a>(&'a self) -> impl Iterator<Item=&'a Type> {
        self.1.iter()
    }
}

impl darling::FromMeta for MultiformExpr {
    fn from_list(items: &[NestedMeta]) -> Result<Self, darling::Error> {
        if let [ref item] = items[..] {
            match item {
                NestedMeta::Lit(lit) => Self::from_value(lit),
                NestedMeta::Meta(meta) => match meta {
                    Meta::Path(path) => Ok(MultiformExpr(path.to_token_stream())),
                    _ => Err(
                        darling::Error::custom("Unsupported meta type")
                            .with_span(item)
                    )
                }
            }
        } else {
            Err(
                darling::Error::custom("")
                    .with_span(&items[0])
            )
        }
    }

    fn from_value(value: &Lit) -> Result<Self, darling::Error> {
        match value {
            Lit::Str(s) => Ok(MultiformExpr(TokenStream::from_str(&s.value()).unwrap())),
            Lit::Int(i) => Ok(MultiformExpr(i.to_token_stream())),
            Lit::ByteStr(b) => Ok(MultiformExpr(quote::quote!{ *#b })),
            _ => Err(
                darling::Error::custom("Unsupported value type")
                    .with_span(value)
            )
        }
    }
}

fn to_tokens(string: Option<MultiformExpr>) -> Option<TokenStream> {
    string.map(|MultiformExpr(tokens)| tokens)
}

fn to_assert(s: Strs) -> Assert {
    match s {
        s if !s.is_list() => Assert(TokenStream::from_str(&s.single()).unwrap(), None),
        _ => {
            if let &[assertion, err] = &s.multiple()[..] {
                Assert(
                    TokenStream::from_str(assertion).unwrap(),
                    Some(TokenStream::from_str(err).unwrap())
                )
            } else {
                panic!("Bad format for assert")
            }
        }
    }
}

#[derive(Debug)]
enum Strs {
    WrappedStr(String),
    WrappedStrList(Vec<String>)
}

use Strs::*;

impl Strs {
    fn single(&self) -> &str {
        match self {
            WrappedStr(s) => s,
            WrappedStrList(s) => &s[0]
        }
    }

    fn multiple(&self) -> Vec<&str> {
        match self {
            WrappedStr(s) => vec![s],
            WrappedStrList(s) => s.iter().map(|s| &**s).collect()
        }
    }

    fn is_list(&self) -> bool {
        if let WrappedStrList(x) = self {
            x.len() != 1
        } else {
            false
        }
    }
}

impl darling::FromMeta for Strs {
    fn from_nested_meta(item: &NestedMeta) -> Result<Self, darling::Error> {
        match item {
            NestedMeta::Lit(Lit::Str(lstr)) => Ok(WrappedStr(lstr.value())),
            _ => {
                Err(darling::Error::custom("Improper formatting"))
            },
        }
    }

    fn from_string(value: &str) -> Result<Self, darling::Error> {
        Ok(WrappedStr(value.to_string()))
    }

    fn from_list(items: &[NestedMeta]) -> Result<Self, darling::Error> {
        items.iter()
            .map(|item|{
                match item {
                    NestedMeta::Lit(Lit::Str(lstr)) => Ok(lstr.value()),
                    _ => {
                        Err(darling::Error::custom("Improper formatting"))
                    },
                }
            })
            .collect::<Result<Vec<String>, _>>()
            .map(|x| WrappedStrList(x))
    }
}

impl syn::parse::Parse for ParseField {
    fn parse(input: syn::parse::ParseStream) -> Result<Self, syn::Error> {
        Ok(Self(Field::parse_named(input)?))
    }
} 

impl darling::FromMeta for Imports {
    fn from_list(items: &[NestedMeta]) -> Result<Self, darling::Error> {
        let (idents, types) = 
            items.into_iter()
                .map(|item| -> Result<(Ident, Type), darling::Error> {
                    match item {
                        NestedMeta::Lit(Lit::Str(s)) => {
                            let ParseField(Field { ident, ty, .. }) = syn::parse_str(&s.value()).unwrap();
                            Ok((ident.unwrap(), ty))
                        }
                        _ => Err(darling::Error::custom("Incorrect format for imports, should be list of strings"))
                    }
                })
                .collect::<Result<Vec<(_, _)>, _>>()?
                .into_iter()
                .unzip();
        Ok(Imports(idents, types))
    }
}

impl darling::FromMeta for PassedValues {
    fn from_list(items: &[NestedMeta]) -> Result<Self, darling::Error> {
        Ok(PassedValues(items.into_iter().map(|item|{
            Ok(match item {
                NestedMeta::Meta(Meta::Path(path)) => path.into_token_stream(),
                NestedMeta::Lit(Lit::Str(s))
                    => syn::parse_str::<Expr>(&s.value()).unwrap().into_token_stream(),
                NestedMeta::Lit(lit)
                    => lit.into_token_stream(),
                _ => Err(darling::Error::custom("Passed values can only contains Paths and literals"))?
            })
        }).collect::<Result<_, _>>()?))
    }
}
