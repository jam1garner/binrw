use crate::codegen::typed_builder::{Builder, BuilderField};
use crate::parser::meta_types::IdentTypeMaybeDefault;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::Ident;

use crate::parser::Imports;

impl Imports {
    pub(crate) fn destructure(
        &self,
        type_name: Option<&Ident>,
        is_write: bool,
    ) -> Option<TokenStream> {
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
            Imports::Raw(ident, _) => Some(quote! {
                mut #ident
            }),
            Imports::Named(args) => type_name.map(|type_name| {
                let args_ty_name = arg_type_name(type_name, is_write);
                let idents = args.iter().map(|x| &x.ident);
                quote! {
                    #args_ty_name {
                        #(#idents),*
                    }
                }
            }),
        }
    }

    pub(crate) fn args_type(
        &self,
        type_name: &Ident,
        ty_vis: &syn::Visibility,
        is_write: bool,
    ) -> (TokenStream, Option<TokenStream>) {
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
            Imports::Raw(_, ty) => (ty.to_token_stream(), None),
            Imports::Named(args) => generate_named_arg_type(type_name, ty_vis, args, is_write),
        }
    }
}

fn arg_type_name(ty_name: &Ident, is_write: bool) -> Ident {
    if is_write {
        format_ident!("{}BinWriteArgs", ty_name, span = Span::mixed_site())
    } else {
        format_ident!("{}BinReadArgs", ty_name, span = Span::mixed_site())
    }
}

fn generate_named_arg_type(
    ty_name: &Ident,
    vis: &syn::Visibility,
    args: &[IdentTypeMaybeDefault],
    is_write: bool,
) -> (TokenStream, Option<TokenStream>) {
    let fields: Vec<BuilderField> = args.iter().map(Into::into).collect();

    let builder_ident = if is_write {
        format_ident!("{}BinWriteArgBuilder", ty_name, span = Span::mixed_site())
    } else {
        format_ident!("{}BinReadArgBuilder", ty_name, span = Span::mixed_site())
    };
    let result_name = arg_type_name(ty_name, is_write);

    let type_definition = Builder {
        builder_name: &builder_ident,
        result_name: &result_name,
        fields: &fields,
        generics: &[],
        vis,
    }
    .generate(true);

    (result_name.to_token_stream(), Some(type_definition))
}
