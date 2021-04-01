use crate::codegen::typed_builder::{Builder, BuilderField};
use crate::parser::{attrs, meta_types::NamedImport, KeywordToken, TrySet};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Type};

#[derive(Debug, Clone)]
pub(crate) enum Imports {
    None,
    #[allow(dead_code)]
    List(Vec<Ident>, Vec<Type>),
    Tuple(Ident, Box<Type>),
    Named(Vec<NamedImport>),
}

impl Default for Imports {
    fn default() -> Self {
        Imports::None
    }
}

impl Imports {
    pub fn destructure(&self, type_name: Option<&Ident>) -> Option<TokenStream> {
        match self {
            Imports::None => None,
            Imports::List(idents, _) => {
                if idents.is_empty() {
                    None
                } else {
                    let idents = idents.iter();
                    Some(quote! {
                        (#(mut #idents,)*)
                    })
                }
            }
            Imports::Tuple(ident, _) => Some(quote! {
                mut #ident
            }),
            Imports::Named(args) => type_name.map(|type_name| {
                let args_ty_name = arg_type_name(type_name);
                let idents = args.iter().map(|x| &x.ident);
                quote! {
                    #args_ty_name {
                        #(#idents),*
                    }
                }
            }),
        }
    }

    pub fn args_type(&self, type_name: &Ident) -> (TokenStream, Option<TokenStream>) {
        match self {
            Imports::None => (quote! { () }, None),
            Imports::List(_, types) => {
                let types = types.iter();
                (
                    quote! {
                        (#(#types,)*)
                    },
                    None,
                )
            }
            Imports::Tuple(_, ty) => (ty.to_token_stream(), None),
            Imports::Named(args) => generate_named_arg_type(type_name, args),
        }
    }
}

fn arg_type_name(ty_name: &Ident) -> Ident {
    format_ident!("{}BinReadArgs", ty_name, span = Span::mixed_site())
}

fn generate_named_arg_type(
    ty_name: &Ident,
    args: &[NamedImport],
) -> (TokenStream, Option<TokenStream>) {
    let fields: Vec<BuilderField> = args.iter().map(Into::into).collect();

    let builder_ident = format_ident!("{}BinReadArgBuilder", ty_name, span = Span::mixed_site());
    let result_name = arg_type_name(ty_name);

    let type_definition = Builder {
        builder_name: &builder_ident,
        result_name: &result_name,
        fields: &fields,
    }
    .generate();

    (result_name.to_token_stream(), Some(type_definition))
}

impl From<attrs::Import> for Imports {
    fn from(value: attrs::Import) -> Self {
        if value.fields.is_empty() {
            Self::None
        } else {
            Self::Named(value.fields.iter().cloned().collect())
        }
    }
}

impl From<attrs::ImportTuple> for Imports {
    fn from(value: attrs::ImportTuple) -> Self {
        Imports::Tuple(value.value.ident, value.value.ty.into())
    }
}

impl<T: Into<Imports> + KeywordToken> TrySet<Imports> for T {
    fn try_set(self, to: &mut Imports) -> syn::Result<()> {
        if matches!(*to, Imports::None) {
            *to = self.into();
            Ok(())
        } else {
            Err(syn::Error::new(
                self.keyword_span(),
                "conflicting import keyword",
            ))
        }
    }
}
