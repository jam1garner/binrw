use crate::parser::{KeywordToken, TrySet, attrs};
use proc_macro2::TokenStream;
use syn::{Ident, Type};
use quote::{ToTokens, quote};

#[derive(Debug, Clone)]
pub(crate) enum Imports {
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
            Imports::None => quote! { () },
            Imports::List(idents, _) => {
                let idents = idents.iter();
                quote! {
                    (#(mut #idents,)*)
                }
            },
            Imports::Tuple(ident, _) => quote! {
                mut #ident
            }
        }
    }

    pub fn types(&self) -> TokenStream {
        match self {
            Imports::None => quote! { () },
            Imports::List(_, types) => {
                let types = types.iter();
                quote! {
                    (#(#types,)*)
                }
            },
            Imports::Tuple(_, ty) => {
                ty.to_token_stream()
            }
        }
    }
}

impl From<attrs::Import> for Imports {
    fn from(value: attrs::Import) -> Self {
        let (idents, tys): (Vec<_>, Vec<_>) = value.fields
        .iter()
        .cloned()
        .map(|import_arg| (import_arg.ident, import_arg.ty))
        .unzip();
        Self::List(idents, tys)
    }
}

impl From<attrs::ImportTuple> for Imports {
    fn from(value: attrs::ImportTuple) -> Self {
        Imports::Tuple(value.arg.ident, value.arg.ty.into())
    }
}

impl <T: Into<Imports> + KeywordToken> TrySet<Imports> for T {
    fn try_set(self, to: &mut Imports) -> syn::Result<()> {
        if matches!(*to, Imports::None) {
            *to = self.into();
            Ok(())
        } else {
            Err(syn::Error::new(self.keyword_span(), "duplicate import keyword"))
        }
    }
}
