use crate::codegen::typed_builder::{Builder, BuilderField};
use crate::parser::meta_types::IdentTypeMaybeDefault;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::Ident;

use crate::parser::Imports;

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
            Imports::Raw(ident, _) => Some(quote! {
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
            Imports::Raw(_, ty) => (ty.to_token_stream(), None),
            Imports::Named(args) => generate_named_arg_type(type_name, args),
        }
    }
}

fn arg_type_name(ty_name: &Ident) -> Ident {
    format_ident!("{}BinReadArgs", ty_name, span = Span::mixed_site())
}

fn generate_named_arg_type(
    ty_name: &Ident,
    args: &[IdentTypeMaybeDefault],
) -> (TokenStream, Option<TokenStream>) {
    let fields: Vec<BuilderField> = args.iter().map(Into::into).collect();

    let builder_ident = format_ident!("{}BinReadArgBuilder", ty_name, span = Span::mixed_site());
    let result_name = arg_type_name(ty_name);

    let type_definition = Builder {
        builder_name: &builder_ident,
        result_name: &result_name,
        fields: &fields,
        generics: &[]
    }
    .generate(true);

    (result_name.to_token_stream(), Some(type_definition))
}
