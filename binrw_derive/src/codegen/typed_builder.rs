use syn::{Ident, Type};
use quote::quote;
use proc_macro2::TokenStream;

use crate::codegen::sanitization::*;

pub(crate) enum BuilderFieldKind {
    Required,
    Optional {
        default: syn::Expr,
    },
}

pub(crate) struct BuilderField {
    pub(crate) name: Ident,
    pub(crate) ty: Type,
    pub(crate) kind: BuilderFieldKind,
}

pub(crate) struct Builder<'a> {
    pub(crate) builder_name: &'a Ident,
    pub(crate) result_name: &'a Ident,
    pub(crate) fields: &'a [BuilderField],
}

impl<'a> Builder<'a> {
    pub(crate) fn generate(&self) -> TokenStream {
        let builder_name = self.builder_name;
        let name = self.result_name;
        let fields = self.generate_fields();
        let initial = self.generate_builder_initial();
        let generics = self.generate_generics();
        let initial_generics = self.generate_initial_generics();
        let satisfied = std::iter::repeat(SATISFIED_OR_OPTIONAL);
        quote!(
            pub struct #name {
                #fields
            }

            impl #name {
                pub fn builder() -> #builder_name < #( #initial_generics ),* > {
                    #initial
                }
            }

            pub struct #builder_name < #( #generics ),* > {
                #fields
                __bind_generics: ::core::marker::PhantomData<( #( #generics ),* )>
            }

            impl< #( #generics : #satisfied ),* > #builder_name < #( #generics ),* > {
                pub fn finalize(self) -> #name {
                    todo!()
                }
            }
        )
    }

    fn generate_fields(&self) -> TokenStream {
        let fields = self.fields.iter().map(|field| field.generate_result_field());
        quote!(
            #( #fields )*
        )
    }

    fn generate_generics(&self) -> Vec<Ident> {
        self.fields
            .iter()
            .map(BuilderField::as_generic)
            .collect()
    }

    fn generate_builder_initial(&self) -> TokenStream {
        let name = self.builder_name;
        let defaults = self.fields.iter().map(BuilderField::initial_value);
        quote!(
            #name {
                #( #defaults )*
                __bind_generics: ::core::marker::PhantomData
            }
        )
    }

    fn generate_initial_generics(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(BuilderField::initial_generic)
            .collect()
    }
}

impl BuilderField {
    fn generate_result_field(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        let ty = match self.kind {
            BuilderFieldKind::Required => quote!(Option<#ty>),
            BuilderFieldKind::Optional { .. } => quote!(#ty),
        };
        quote!(
            #name: #ty,
        )
    }

    fn as_generic(&self) -> Ident {
        quote::format_ident!("Field_{}", self.name)
    }

    fn initial_value(&self) -> TokenStream {
        let name = &self.name;
        match self.kind {
            BuilderFieldKind::Required => quote!(
                #name: None,
            ),
            BuilderFieldKind::Optional { ref default } => quote!(
                #name: #default,
            )
        }
    }

    fn initial_generic(&self) -> TokenStream {
        match self.kind {
            BuilderFieldKind::Required => quote!( #NEEDED ),
            BuilderFieldKind::Optional { .. } => quote!( #OPTIONAL ),
        }
    }
}
