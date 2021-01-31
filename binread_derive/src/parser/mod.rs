#[macro_use]
pub(crate) mod macros;
mod field_level_attrs;
mod keywords;
pub(crate) mod meta_types;
#[cfg(test)]
mod parsing_tests;
mod top_level_attrs;

pub(crate) use field_level_attrs::{CondEndian, FieldLevelAttrs, Map};
use proc_macro2::TokenStream;
use syn::{Expr, Ident, parse::Parse, Type, spanned::Spanned};
use quote::ToTokens;
use self::meta_types::{MetaAttrList, MetaList, MetaValue};
pub(crate) use top_level_attrs::{EnumErrorHandling, TopLevelAttrs};

pub(crate) trait KeywordToken {
    fn display() -> &'static str;
    fn dyn_display(&self) -> &'static str {
        Self::display()
    }
}

impl <T: syn::token::Token> KeywordToken for T {
    fn display() -> &'static str {
        <Self as syn::token::Token>::display()
    }
}

pub(crate) fn duplicate_attr<Keyword: KeywordToken + Spanned, R>(kw: &Keyword) -> syn::Result<R> {
    Err(syn::Error::new(kw.span(), format!("duplicate {} attribute", KeywordToken::dyn_display(kw))))
}

pub(crate) trait FromAttrs<Attr: syn::parse::Parse> {
    fn try_from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> where Self: Default + Sized {
        #[allow(clippy::filter_map)]
        let attrs = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("br") || attr.path.is_ident("binread"))
            .flat_map(|attr| {
                match syn::parse2::<MetaAttrList<Attr>>(attr.tokens.clone()) {
                    Ok(list) => either::Either::Right(list.into_iter().map(Ok)),
                    Err(err) => either::Either::Left(core::iter::once(Err(err))),
                }
            });

        let mut this = Self::default();
        let mut all_errors = None::<syn::Error>;
        for attr in attrs {
            let result = match attr {
                Ok(attr) => this.try_set_attr(attr),
                Err(e) => Err(e),
            };

            if let Err(parse_error) = result {
                if let Some(error) = &mut all_errors {
                    error.combine(parse_error);
                } else {
                    all_errors = Some(parse_error);
                }
            }
        }

        match all_errors {
            Some(error) => Err(error),
            None => Ok(this),
        }
    }

    fn try_set_attr(&mut self, attr: Attr) -> syn::Result<()>;
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

pub(crate) fn set_option_ts<K, V>(value: &mut Option<TokenStream>, attr: &MetaValue<K, V>) -> syn::Result<()>
    where K: KeywordToken + Spanned,
          V: ToTokens,
{
    if value.is_some() {
        duplicate_attr(&attr.ident)
    } else {
        *value = Some(attr.value.to_token_stream());
        Ok(())
    }
}
