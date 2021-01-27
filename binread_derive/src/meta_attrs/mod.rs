mod parser;
mod spanned_value;
mod top_level_attrs;
mod field_level_attrs;
pub(crate) use top_level_attrs::{EnumErrorHandling, TopLevelAttrs};
pub(crate) use field_level_attrs::{CondEndian, FieldLevelAttrs, Map};

use proc_macro2::TokenStream;
use syn::{Expr, Ident, parse::Parse, Type, spanned::Spanned};
use quote::ToTokens;

use self::parser::{MetaAttrList, MetaList};

pub(crate) fn collect_attrs<P: Parse>(attrs: &[syn::Attribute]) -> syn::Result<impl Iterator<Item = P>> {
    Ok(attrs
        .iter()
        .filter_map(|attr|
            if attr.path.is_ident("br") || attr.path.is_ident("binread") {
                Some(syn::parse2::<MetaAttrList<P>>(attr.tokens.clone()))
            } else {
                None
            })
        // TODO: Do not collect, iterate instead
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|list| list.0.into_iter()))
}

#[derive(Debug, Clone)]
pub struct Assert(pub TokenStream, pub Option<TokenStream>);

#[derive(Debug, Clone)]
pub enum PassedArgs {
    None,
    List(Vec<TokenStream>),
    Tuple(TokenStream)
}

impl Default for PassedArgs {
    fn default() -> Self {
        PassedArgs::None
    }
}

#[derive(Debug, Clone)]
pub enum Imports {
    None,
    List(Vec<Ident>, Vec<Type>),
    Tuple(Ident, Box<Type>)
}

impl Default for Imports {
    fn default() -> Self {
        Imports::None
    }
}

impl Imports {
    pub fn idents(&self) -> TokenStream {
        match self {
            Imports::None => quote::quote! { () },
            Imports::List(idents, _) => {
                let idents = idents.iter();
                quote::quote! {
                    (#(mut #idents,)*)
                }
            },
            Imports::Tuple(ident, _) => quote::quote! {
                mut #ident
            }
        }
    }

    pub fn types(&self) -> TokenStream {
        match self {
            Imports::None => quote::quote! { () },
            Imports::List(_, types) => {
                let types = types.iter();
                quote::quote! {
                    (#(#types,)*)
                }
            },
            Imports::Tuple(_, ty) => {
                ty.to_token_stream()
            }
        }
    }

    pub fn is_some(&self) -> bool {
        !matches!(self, Imports::None)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum MagicType {
    Str,
    ByteStr,
    Byte,
    Char,
    Int(String),
    Float,
    Bool,
    Verbatim
}

pub(crate) fn convert_assert<K>(assert: &MetaList<K, Expr>) -> syn::Result<Assert>
    where K: Parse + Spanned,
{
    let (cond, err) = {
        let mut iter = assert.fields.iter();
        let result = (iter.next(), iter.next());
        if iter.next().is_some() {
            return Err(syn::Error::new(
                assert.ident.span(),
                "Too many arguments passed to assert"
            ));
        }
        result
    };

    Ok(Assert(
        cond.into_token_stream(),
        err.map(ToTokens::into_token_stream)
    ))
}
