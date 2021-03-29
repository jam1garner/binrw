use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, Type};

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::meta_types::NamedImport;

pub(crate) enum BuilderFieldKind {
    Required,
    Optional { default: Box<syn::Expr> },
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
        let fields = self.generate_result_fields();
        let builder_fields = self.generate_builder_fields();
        let initial = self.generate_builder_initial();
        let generics = self.generate_generics();
        let initial_generics = self.generate_initial_generics();
        let setters = self.generate_setters();
        let satisfied = std::iter::repeat(SATISFIED_OR_OPTIONAL);
        let field_names: Vec<_> = self.fields.iter().map(|field| &field.name).collect();
        let possible_unwrap = self.fields.iter().map(BuilderField::possible_unwrap);
        quote!(
            #[derive(Clone)]
            pub(crate) struct #name {
                #fields
            }

            impl #name {
                pub fn builder() -> #builder_name < #( #initial_generics ),* > {
                    #initial
                }
            }

            #( #setters )*

            #[allow(non_camel_case_types)]
            pub(crate) struct #builder_name < #( #generics ),* > {
                #builder_fields
                __bind_generics: ::core::marker::PhantomData<( #( #generics ),* )>
            }

            #[allow(non_camel_case_types)]
            impl< #( #generics : #satisfied ),* > #builder_name < #( #generics ),* > {
                pub fn finalize(self) -> #name {
                    let #builder_name {
                        #(
                            #field_names,
                        )*
                        ..
                    } = self;

                    #name {
                        #(
                            #field_names #possible_unwrap,
                        )*
                    }
                }
            }
        )
    }

    fn generate_builder_fields(&self) -> TokenStream {
        let fields = self.fields.iter().map(BuilderField::generate_builder_field);
        quote!(
            #( #fields )*
        )
    }

    fn generate_result_fields(&self) -> TokenStream {
        let fields = self.fields.iter().map(BuilderField::generate_result_field);
        quote!(
            #( #fields )*
        )
    }

    fn generate_generics(&self) -> Vec<Ident> {
        self.fields.iter().map(BuilderField::as_generic).collect()
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

    fn generate_setters(&self) -> Vec<TokenStream> {
        let builder_name = self.builder_name;
        self.fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let generics = self.generate_generics();

                // The current field is not generic
                let mut generic_params = generics.clone();
                generic_params.remove(i);

                // The generics required for the builder should be generic for all parameters
                // except the current field, which is set to its initial state
                let mut required_generics: Vec<_> = generics
                    .into_iter()
                    .map(ToTokens::into_token_stream)
                    .collect();
                required_generics[i] = field.initial_generic();

                // the resulting generics should be the same as before, but with the type for
                // the current field being marked as satisfied.
                let mut resulting_generics = required_generics.clone();
                resulting_generics[i] = quote!( #SATISFIED );

                let field_names: Vec<_> = self.fields.iter().map(|field| &field.name).collect();
                let field_name = &field.name;
                let ty = &field.ty;

                let field_result = match field.kind {
                    BuilderFieldKind::Required => quote!(Some(val)),
                    BuilderFieldKind::Optional { .. } => quote!(val),
                };

                quote!(
                    #[allow(non_camel_case_types)]
                    impl< #( #generic_params ),* > #builder_name < #( #required_generics ),* > {
                        pub fn #field_name(
                            self, val: #ty
                        ) -> #builder_name < #( #resulting_generics ),* > {
                            let #builder_name {
                                #(
                                    #field_names,
                                )*
                                ..
                            } = self;

                            let #field_name = #field_result;

                            #builder_name {
                                #(
                                    #field_names,
                                )*
                                __bind_generics: ::core::marker::PhantomData
                            }
                        }
                    }
                )
            })
            .collect()
    }
}

impl From<&NamedImport> for BuilderField {
    fn from(import: &NamedImport) -> Self {
        let name = import.ident.clone();
        let ty = import.ty.clone();

        // if no default is provided, mark as required
        let kind = import
            .default
            .as_ref()
            .map_or(BuilderFieldKind::Required, |default| {
                BuilderFieldKind::Optional {
                    default: default.clone(),
                }
            });

        BuilderField { name, ty, kind }
    }
}

impl BuilderField {
    fn generate_builder_field(&self) -> TokenStream {
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

    fn generate_result_field(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
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
            ),
        }
    }

    fn initial_generic(&self) -> TokenStream {
        match self.kind {
            BuilderFieldKind::Required => quote!( #NEEDED ),
            BuilderFieldKind::Optional { .. } => quote!( #OPTIONAL ),
        }
    }

    fn possible_unwrap(&self) -> TokenStream {
        let name = &self.name;
        match self.kind {
            BuilderFieldKind::Required => quote!( : #name.unwrap() ),
            BuilderFieldKind::Optional { .. } => quote!(),
        }
    }
}
