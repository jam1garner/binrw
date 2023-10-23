use crate::{
    meta_types::IdentTypeMaybeDefault,
    util::{from_crate, ident_str},
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{GenericArgument, GenericParam, Ident, Type, Visibility};

ident_str! {
    SATISFIED_OR_OPTIONAL = from_crate!(__private::SatisfiedOrOptional);
    SATISFIED = from_crate!(__private::Satisfied);
    NEEDED = from_crate!(__private::Needed);
    OPTIONAL = from_crate!(__private::Optional);
    NAMED_ARGS = from_crate!(NamedArgs);
}

// Lint: Describing which field name is which prevents confusion.
#[cfg_attr(nightly, allow(clippy::struct_field_names))]
pub(super) struct Builder<'a> {
    pub(super) owner_name: Option<&'a Ident>,
    pub(super) builder_name: &'a Ident,
    pub(super) result_name: &'a Ident,
    pub(super) fields: &'a [BuilderField],
    pub(super) generics: &'a [GenericParam],
    pub(super) vis: &'a Visibility,
    pub(super) is_write: bool,
}

impl<'a> Builder<'a> {
    pub(super) fn generate(&self, define_result: bool) -> TokenStream {
        let builder_name = self.builder_name;
        let name = self.result_name;
        let user_bounds = {
            let generics = self.generics;
            quote! { #( #generics, )* }
        };
        let vis = self.vis;
        let user_generic_args = self.user_generic_args();
        let fields = self.generate_result_fields();
        let builder_fields = self.generate_builder_fields();
        let initial = self.generate_builder_initial();
        let generics = self.generate_generics();
        let initial_generics = self.generate_initial_generics();
        let setters = self.generate_setters(&user_generic_args);
        let satisfied = {
            let satisfied = SATISFIED_OR_OPTIONAL;
            quote! {
                #( #generics : #satisfied ),*
            }
        };
        let field_names = self.fields.iter().map(|field| &field.name);
        let possible_unwrap = self.fields.iter().map(BuilderField::possible_unwrap);
        let optional_finalizers = self.optional_finalizers();
        let generics = quote! { #( #generics ),* };

        let res_struct = if define_result {
            let docs = self.owner_name.map(|owner_name| {
                let (impl_name, impl_fn) = if self.is_write {
                    ("BinWrite", "write_options")
                } else {
                    ("BinRead", "read_options")
                };
                format!(
                    "Named arguments for the [`{impl_name}::{impl_fn}`](::binrw::{impl_name}::{impl_fn}) implementation of [`{owner_name}`].",
                )
            });

            let derives = if self.are_all_fields_optional() {
                quote!(#[derive(Clone, Default)])
            } else {
                quote!(#[derive(Clone)])
            };
            Some(quote!(
                #derives
                #[doc = #docs]
                #vis struct #name < #user_bounds > {
                    #fields
                }
            ))
        } else {
            None
        };

        let builder_docs = format!(
            "A builder for [`{name}`] objects. Compatible with [`binrw::args!`](::binrw::args)."
        );

        quote!(
            #res_struct

            impl< #user_bounds > #name < #user_generic_args > {
                /// Creates a new builder for this type.
                #vis fn builder() -> #builder_name < #user_generic_args #initial_generics > {
                    #initial
                }
            }

            impl< #user_bounds > #NAMED_ARGS for #name < #user_generic_args > {
                type Builder = #builder_name < #user_generic_args #initial_generics >;

                fn builder() -> Self::Builder {
                    Self::builder()
                }
            }

            #( #setters )*

            #[doc = #builder_docs]
            #[allow(non_camel_case_types)]
            #vis struct #builder_name < #user_bounds #generics > {
                #builder_fields
                __bind_generics: ::core::marker::PhantomData<( #generics )>
            }

            #optional_finalizers

            #[allow(non_camel_case_types)]
            impl<
                #user_bounds
                #satisfied
            >
                #builder_name
                <
                    #user_generic_args
                    #generics
                >
            {
                /// Builds the object.
                #vis fn finalize(self) -> #name < #user_generic_args > {
                    let #builder_name {
                        #( #field_names, )*
                        ..
                    } = self;

                    #name {
                        #( #possible_unwrap, )*
                    }
                }
            }
        )
    }

    fn user_generic_args(&self) -> TokenStream {
        let args = self.generics.iter().map(|generic| match generic {
            GenericParam::Type(ty) => GenericArgument::Type(Type::Path(syn::TypePath {
                qself: None,
                path: ty.ident.clone().into(),
            })),
            GenericParam::Const(cnst) => GenericArgument::Const(syn::Expr::Path(syn::ExprPath {
                attrs: Vec::new(),
                qself: None,
                path: cnst.ident.clone().into(),
            })),
            GenericParam::Lifetime(lt) => GenericArgument::Lifetime(lt.lifetime.clone()),
        });

        quote! { #(#args,)* }
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

    fn generate_initial_generics(&self) -> TokenStream {
        let generics = self.fields.iter().map(BuilderField::initial_generic);
        quote! { #(#generics,)* }
    }

    fn generate_setters<'builder>(
        &'builder self,
        user_generic_args: &'builder TokenStream,
    ) -> impl Iterator<Item = TokenStream> + 'builder {
        let builder_name = self.builder_name;
        let user_bounds = self.generics;
        self.fields.iter().enumerate().map(move |(i, field)| {
            let generics = self.generate_generics();
            let vis = self.vis;

            // The current field is not generic
            let generic_params = generics
                .iter()
                .enumerate()
                .filter_map(|(n, p)| (n != i).then_some(p));

            // The generics required for the builder should be generic for all parameters
            // except the current field, which is set to its initial state
            let required_generics = generics.iter().enumerate().map(|(n, t)| {
                if n == i {
                    field.initial_generic()
                } else {
                    t.to_token_stream()
                }
            });

            // the resulting generics should be the same as before, but with the type for
            // the current field being marked as satisfied.
            let resulting_generics = generics.iter().enumerate().map(|(n, t)| {
                if n == i {
                    SATISFIED.to_token_stream()
                } else {
                    t.to_token_stream()
                }
            });

            let field_names = {
                let names = self.fields.iter().map(|field| &field.name);
                quote! { #( #names, )* }
            };
            let field_name = &field.name;
            let ty = &field.ty;
            let docs = format!("Sets `{field_name}` to the given value.");

            let field_result = match field.kind {
                BuilderFieldKind::Required | BuilderFieldKind::TryOptional => quote!(Some(val)),
                BuilderFieldKind::Optional { .. } => quote!(val),
            };

            quote!(
                #[allow(non_camel_case_types, unused_variables)]
                impl<
                    #( #user_bounds, )*
                    #( #generic_params ),*
                > #builder_name < #user_generic_args #( #required_generics ),* > {
                    #[doc = #docs]
                    #vis fn #field_name(
                        self, val: #ty
                    ) -> #builder_name < #user_generic_args #( #resulting_generics ),* > {
                        let #builder_name {
                            #field_names
                            ..
                        } = self;

                        let #field_name = #field_result;

                        #builder_name {
                            #field_names
                            __bind_generics: ::core::marker::PhantomData
                        }
                    }
                }
            )
        })
    }

    fn are_all_fields_optional(&self) -> bool {
        self.fields
            .iter()
            .all(|field| matches!(field.kind, BuilderFieldKind::Optional { .. }))
    }

    fn optional_finalizers(&self) -> TokenStream {
        if !self
            .fields
            .iter()
            .any(|field| matches!(field.kind, BuilderFieldKind::TryOptional))
        {
            return <_>::default();
        }
        let builder_name = self.builder_name;
        let name = self.result_name;
        let user_bounds = self.generics;
        let vis = self.vis;
        let user_generic_args = self.user_generic_args();
        let generics = self.generate_generics();
        let field_names = {
            let names = self.fields.iter().map(|field| &field.name);
            quote! { #(#names,)* }
        };
        let possible_unwrap = {
            let unwraps = self
                .fields
                .iter()
                .map(BuilderField::possible_unwrap_or_default);
            quote! { #(#unwraps,)* }
        };

        let finalizers = self
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| matches!(field.kind, BuilderFieldKind::TryOptional))
            .map(|(i, field)| {
                let current_field_ty = &field.ty;
                let satisfied_generics = generics.iter().enumerate().map(|(n, generic)| {
                    if i == n {
                        quote!(#NEEDED)
                    } else {
                        quote!(#generic)
                    }
                });
                let filtered_generics = generics.iter().enumerate().filter_map(|(n, generic)| {
                    if i == n {
                        None
                    } else {
                        Some(quote!(#generic : #SATISFIED_OR_OPTIONAL))
                    }
                });

                quote! {
                    #[allow(non_camel_case_types)]
                    impl<
                        #( #user_bounds, )*
                        #( #filtered_generics ),*
                    >
                        #builder_name
                        <
                            #user_generic_args
                            #( #satisfied_generics ),*
                        >
                    where
                        #current_field_ty: Default,
                    {
                        /// Builds the object.
                        #vis fn finalize(self) -> #name < #user_generic_args > {
                            let #builder_name {
                                #field_names
                                ..
                            } = self;

                            #name {
                                #possible_unwrap
                            }
                        }
                    }
                }
            });

        quote! { #(#finalizers)* }
    }
}

pub(super) struct BuilderField {
    pub(super) name: Ident,
    pub(super) ty: Type,
    pub(super) kind: BuilderFieldKind,
}

impl BuilderField {
    fn generate_builder_field(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        let ty = match self.kind {
            BuilderFieldKind::Required | BuilderFieldKind::TryOptional => quote!(Option<#ty>),
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
            BuilderFieldKind::Required | BuilderFieldKind::TryOptional => quote!(
                #name: None,
            ),
            BuilderFieldKind::Optional { ref default } => quote!(
                #name: #default,
            ),
        }
    }

    fn initial_generic(&self) -> TokenStream {
        match self.kind {
            BuilderFieldKind::Required | BuilderFieldKind::TryOptional => quote!( #NEEDED ),
            BuilderFieldKind::Optional { .. } => quote!( #OPTIONAL ),
        }
    }

    fn possible_unwrap(&self) -> TokenStream {
        let name = &self.name;
        match self.kind {
            BuilderFieldKind::Required | BuilderFieldKind::TryOptional => {
                quote! { #name: #name.unwrap() }
            }
            BuilderFieldKind::Optional { .. } => quote! { #name },
        }
    }

    fn possible_unwrap_or_default(&self) -> TokenStream {
        let name = &self.name;
        match self.kind {
            BuilderFieldKind::Required => quote!( #name: #name.unwrap() ),
            BuilderFieldKind::Optional { .. } => quote! { #name },
            BuilderFieldKind::TryOptional => quote! { #name: #name.unwrap_or_default() },
        }
    }
}

impl From<IdentTypeMaybeDefault> for BuilderField {
    fn from(import: IdentTypeMaybeDefault) -> Self {
        let name = import.ident;
        let ty = import.ty;

        // if no default is provided, mark as required
        let kind = import
            .default
            .map_or(BuilderFieldKind::Required, |default| {
                BuilderFieldKind::Optional { default }
            });

        BuilderField { name, ty, kind }
    }
}

pub(super) enum BuilderFieldKind {
    Required,
    TryOptional,
    Optional { default: Box<syn::Expr> },
}
