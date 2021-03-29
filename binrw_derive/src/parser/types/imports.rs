use crate::codegen::sanitization::TRAIT_NAME;
use crate::codegen::typed_builder::{Builder, BuilderField, BuilderFieldKind};
use crate::parser::{attrs, KeywordToken, TrySet};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Type};

#[derive(Debug, Clone)]
pub(crate) enum Imports {
    None,
    List(Vec<Ident>, Vec<Type>),
    Tuple(Ident, Box<Type>),
    Named(Vec<Ident>, Vec<Type>),
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
            Imports::Named(idents, _) => type_name.map(|type_name| {
                let args_ty_name = arg_type_name(type_name);
                quote! {
                    #args_ty_name {
                        #(#idents),*
                    }
                }
            }),
        }
    }

    pub fn args_type(&self, type_name: &Ident) -> (TokenStream, TokenStream) {
        let empty = quote! {};
        match self {
            Imports::None => (quote! { () }, empty),
            Imports::List(_, types) => {
                let types = types.iter();
                (
                    quote! {
                        (#(#types,)*)
                    },
                    empty,
                )
            }
            Imports::Tuple(_, ty) => (ty.to_token_stream(), empty),
            Imports::Named(names, tys) => generate_named_arg_type(type_name, names, tys),
        }
    }
}

fn arg_type_name(ty_name: &Ident) -> Ident {
    format_ident!("{}BinReadArgs", ty_name, span = Span::mixed_site())
}

fn generate_named_arg_type(
    ty_name: &Ident,
    names: &[Ident],
    tys: &[Type],
) -> (TokenStream, TokenStream) {
    let fields: Vec<BuilderField> = names
        .iter()
        .zip(tys.iter())
        .map(|(name, ty)| BuilderField {
            name: name.clone(),
            ty: ty.clone(),
            kind: BuilderFieldKind::Required,
        })
        .collect();

    let builder_ident = format_ident!("{}BinReadArgBuilder", ty_name, span = Span::mixed_site());
    let result_name = arg_type_name(ty_name);

    let type_definition = Builder {
        builder_name: &builder_ident,
        result_name: &result_name,
        fields: &fields,
    }
    .generate();

    (result_name.to_token_stream(), type_definition)
}

impl From<attrs::Import> for Imports {
    fn from(value: attrs::Import) -> Self {
        if value.fields.is_empty() {
            Self::None
        } else {
            let (idents, tys) = value
                .fields
                .iter()
                .cloned()
                .map(|import_arg| (import_arg.ident, import_arg.ty))
                .unzip();

            // Change this to Self::List to use old tuple args
            Self::Named(idents, tys)
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
