use syn::{Ident, Type};
use quote::quote;
use proc_macro2::TokenStream;

use crate::codegen::sanitization::*;

enum BuilderFieldKind {
    Required,
    Optional {
        default: syn::Expr,
    },
}

struct BuilderField {
    name: Ident,
    ty: Type,
    kind: BuilderFieldKind,
}

struct Builder<'a> {
    builder_name: &'a Ident,
    result_name: &'a Ident,
    fields: &'a [BuilderField],
}

impl<'a> Builder<'a> {
    fn generate(&self) -> TokenStream {
        let builder_name = self.builder_name;
        let name = self.result_name;
        let fields = self.generate_fields();
        let initial = self.generate_builder_initial();
        let generics = self.generate_generics();
        let satisfied = std::iter::repeat(SATISFIED_OR_OPTIONAL);
        quote!(
            struct #name {
                #fields
            }

            impl #name {
                fn builder() -> #builder_name {
                    #initial
                }
            }

            struct #builder_name < #( #generics ),* > {
                #fields,
                __bind_generics: ::core::marker::PhantomData<( #( #generics ),* )>,
            }

            impl< #( #generics : #satisfied ),* > #builder_name < #( #generics ),* > {
                fn finalize(self) -> #name {

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
            }
        )
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
}
