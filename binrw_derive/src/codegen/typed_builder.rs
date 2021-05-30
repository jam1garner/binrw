use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{GenericArgument, GenericParam, Ident, Type, Visibility};

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::meta_types::IdentTypeMaybeDefault;

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
    pub(crate) generics: &'a [GenericParam],
    pub(crate) vis: &'a Visibility,
}

impl<'a> Builder<'a> {
    pub(crate) fn generate(&self, define_result: bool) -> TokenStream {
        let builder_name = self.builder_name;
        let name = self.result_name;
        let user_bounds = self.generics;
        let vis = self.vis;
        let user_generic_args = self.user_generic_args();
        let fields = self.generate_result_fields();
        let builder_fields = self.generate_builder_fields();
        let initial = self.generate_builder_initial();
        let generics = self.generate_generics();
        let initial_generics = self.generate_initial_generics();
        let setters = self.generate_setters(&user_generic_args);
        let satisfied = &SATISFIED_OR_OPTIONAL;
        let field_names: Vec<_> = self.fields.iter().map(|field| &field.name).collect();
        let possible_unwrap = self.fields.iter().map(BuilderField::possible_unwrap);

        let res_struct = if define_result {
            let derives = if self.are_all_fields_optional() {
                quote!(#[derive(Clone, Default)])
            } else {
                quote!(#[derive(Clone)])
            };
            Some(quote!(
                #derives
                #vis struct #name < #( #user_bounds ),* > {
                    #fields
                }
            ))
        } else {
            None
        };

        quote!(
            #res_struct

            impl< #( #user_bounds ),* > #name < #( #user_generic_args ),* >  {
                /// An inherent method version of [`BinrwNamedArgs`](::binrw::BinrwNamedArgs),
                /// designed for use with [`binrw::args!`](::binrw::args).
                #vis fn builder() -> #builder_name < #( #user_generic_args, )* #( #initial_generics ),* > {
                    #initial
                }
            }

            impl< #( #user_bounds ),* > #BINRW_NAMED_ARGS for #name < #( #user_generic_args ),* > {
                type Builder = #builder_name < #( #user_generic_args, )* #( #initial_generics ),* >;

                fn builder() -> Self::Builder {
                    Self::builder()
                }
            }

            #( #setters )*

            /// A builder for constructing the given type using [`binrw::args!`](::binrw::args).
            #[allow(non_camel_case_types)]
            #vis struct #builder_name <#(#user_bounds,)* #( #generics ),* > {
                #builder_fields
                __bind_generics: ::core::marker::PhantomData<( #( #generics ),* )>
            }

            #[allow(non_camel_case_types)]
            impl<
                #( #user_bounds, )*
                #( #generics : #satisfied ),*
            >
                #builder_name
                <
                    #(#user_generic_args,)*
                    #( #generics ),*
                >
            {
                /// A method to finalize the struct after all builder required fields have been
                /// fulfilled.
                #vis fn finalize(self) -> #name < #(#user_generic_args),* > {
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

    fn user_generic_args(&self) -> Vec<GenericArgument> {
        self.generics
            .iter()
            .map(|generic| match generic {
                GenericParam::Type(ty) => GenericArgument::Type(Type::Path(syn::TypePath {
                    qself: None,
                    path: ty.ident.clone().into(),
                })),
                GenericParam::Const(cnst) => {
                    GenericArgument::Const(syn::Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: cnst.ident.clone().into(),
                    }))
                }
                GenericParam::Lifetime(lt) => GenericArgument::Lifetime(lt.lifetime.clone()),
            })
            .collect()
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

    fn generate_setters<'builder>(
        &'builder self,
        user_generic_args: &'builder [GenericArgument],
    ) -> impl Iterator<Item = TokenStream> + 'builder {
        let builder_name = self.builder_name;
        let user_bounds = self.generics;
        self.fields.iter().enumerate().map(move |(i, field)| {
            let generics = self.generate_generics();
            let vis = self.vis;

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
                #[allow(non_camel_case_types, unused_variables)]
                impl<
                    #( #user_bounds, )*
                    #( #generic_params ),*
                > #builder_name < #( #user_generic_args, )* #( #required_generics ),* > {
                    /// A method to allow this field to be set using the [`binrw::args`]
                    /// macro.
                    ///
                    /// [`binrw::args`]: ::binrw::args
                    #vis fn #field_name(
                        self, val: #ty
                    ) -> #builder_name < #( #user_generic_args, )* #( #resulting_generics ),* > {
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
    }

    fn are_all_fields_optional(&self) -> bool {
        self.fields.iter().all(|field| match field.kind {
            BuilderFieldKind::Optional { .. } => true,
            _ => false,
        })
    }
}

impl From<&IdentTypeMaybeDefault> for BuilderField {
    fn from(import: &IdentTypeMaybeDefault) -> Self {
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
