mod meta;
mod read_options;
pub(crate) mod sanitization;
mod write_options;

use crate::{
    binrw::parser::{
        Assert, AssertionError, CondEndian, Imports, Input, ParseResult, PassedArgs, StructField,
    },
    named_args::{arg_type_name, derive_from_imports},
    util::{quote_spanned_any, IdentStr},
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use sanitization::{
    ARGS, ARGS_LIFETIME, ARGS_MACRO, ASSERT, ASSERT_ERROR_FN, BINREAD_TRAIT, BINWRITE_TRAIT,
    BIN_ERROR, BIN_RESULT, ENDIAN_ENUM, OPT, POS, READER, READ_TRAIT, SEEK_TRAIT, TEMP, WRITER,
    WRITE_TRAIT,
};
use syn::{spanned::Spanned, DeriveInput, Ident, Token, Type, WhereClause};

pub(crate) fn generate_impl<const WRITE: bool>(
    derive_input: &DeriveInput,
    binrw_input: &ParseResult<Input>,
) -> TokenStream {
    let (arg_type, arg_type_declaration) = match binrw_input {
        ParseResult::Ok(binrw_input) | ParseResult::Partial(binrw_input, _) => generate_imports(
            binrw_input.imports(),
            &derive_input.ident,
            &derive_input.vis,
            WRITE,
        ),
        ParseResult::Err(_) => (quote! { () }, None),
    };

    let trait_impl = generate_trait_impl::<WRITE>(binrw_input, derive_input, &arg_type);

    let meta_impls = match binrw_input {
        ParseResult::Ok(binrw_input) | ParseResult::Partial(binrw_input, _) => {
            Some(meta::generate::<WRITE>(binrw_input, derive_input))
        }
        ParseResult::Err(_) => None,
    };

    quote! {
        #trait_impl
        #meta_impls
        #arg_type_declaration
    }
}

fn generate_imports(
    imports: &Imports,
    type_name: &Ident,
    ty_vis: &syn::Visibility,
    is_write: bool,
) -> (TokenStream, Option<TokenStream>) {
    use syn::fold::Fold;

    fn has_elided_lifetime(ty: &syn::Type) -> bool {
        use syn::visit::Visit;
        struct Finder(bool);
        impl Visit<'_> for Finder {
            fn visit_lifetime(&mut self, i: &syn::Lifetime) {
                self.0 |= i.ident == "_";
            }

            fn visit_type_reference(&mut self, i: &syn::TypeReference) {
                self.0 |= i.lifetime.is_none();
            }
        }
        let mut finder = Finder(false);
        finder.visit_type(ty);
        finder.0
    }

    struct ExpandLifetimes;
    impl Fold for ExpandLifetimes {
        fn fold_lifetime(&mut self, mut i: syn::Lifetime) -> syn::Lifetime {
            if i.ident == "_" {
                i.ident = syn::Ident::new(ARGS_LIFETIME, i.ident.span());
            }
            i
        }

        fn fold_type_reference(&mut self, mut i: syn::TypeReference) -> syn::TypeReference {
            if i.lifetime.is_none()
                || matches!(&i.lifetime, Some(lifetime) if lifetime.ident == "_")
            {
                i.lifetime = Some(get_args_lifetime(i.and_token.span()));
            }
            i.elem = Box::new(ExpandLifetimes.fold_type(*i.elem));
            i
        }
    }

    match imports {
        Imports::None => (quote! { () }, None),
        Imports::List(_, types) => {
            let types = types.iter().map(|ty| ExpandLifetimes.fold_type(ty.clone()));
            (quote! { (#(#types,)*) }, None)
        }
        Imports::Raw(_, ty) => (
            ExpandLifetimes
                .fold_type(ty.as_ref().clone())
                .into_token_stream(),
            None,
        ),
        Imports::Named(args) => {
            let name = arg_type_name(type_name, is_write);
            let lifetime = args
                .iter()
                .any(|arg| has_elided_lifetime(&arg.ty))
                .then(|| get_args_lifetime(type_name.span()));
            let defs = derive_from_imports(
                type_name,
                is_write,
                &name,
                ty_vis,
                lifetime.clone(),
                args.iter().map(|arg| {
                    let mut arg = arg.clone();
                    arg.ty = ExpandLifetimes.fold_type(arg.ty);
                    arg
                }),
            );
            (
                if let Some(lifetime) = lifetime {
                    quote_spanned! { type_name.span()=> #name<#lifetime> }
                } else {
                    name.into_token_stream()
                },
                Some(defs),
            )
        }
    }
}

fn generate_trait_impl<const WRITE: bool>(
    binrw_input: &ParseResult<Input>,
    derive_input: &DeriveInput,
    arg_type: &TokenStream,
) -> TokenStream {
    let (trait_name, fn_sig) = if WRITE {
        (
            BINWRITE_TRAIT,
            quote! {
                fn write_options<W: #WRITE_TRAIT + #SEEK_TRAIT>(
                    &self,
                    #WRITER: &mut W,
                    #OPT: #ENDIAN_ENUM,
                    #ARGS: Self::Args<'_>
                ) -> #BIN_RESULT<()>
            },
        )
    } else {
        (
            BINREAD_TRAIT,
            quote! {
                fn read_options<R: #READ_TRAIT + #SEEK_TRAIT>
                    (#READER: &mut R, #OPT: #ENDIAN_ENUM, #ARGS: Self::Args<'_>)
                    -> #BIN_RESULT<Self>
            },
        )
    };

    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    let (fn_impl, where_clause) = match binrw_input {
        ParseResult::Ok(binrw_input) => {
            if WRITE {
                (
                    write_options::generate(binrw_input, derive_input),
                    get_where_clause(binrw_input, where_clause),
                )
            } else {
                (
                    read_options::generate(binrw_input, derive_input),
                    get_where_clause(binrw_input, where_clause),
                )
            }
        }
        // If there is a parsing error, an impl for the trait still needs to be
        // generated to avoid misleading errors at all call sites that use the
        // trait, so emit the trait and just stick the errors inside the generated
        // function
        ParseResult::Partial(_, error) | ParseResult::Err(error) => {
            (error.to_compile_error(), where_clause.cloned())
        }
    };

    let args_lifetime = get_args_lifetime(Span::call_site());
    quote! {
        #[automatically_derived]
        #[allow(non_snake_case, unknown_lints)]
        #[allow(clippy::redundant_closure_call)]
        impl #impl_generics #trait_name for #name #ty_generics #where_clause {
            type Args<#args_lifetime> = #arg_type;

            #fn_sig {
                #fn_impl
            }
        }
    }
}

fn get_where_clause(
    binrw_input: &Input,
    where_clause: Option<&WhereClause>,
) -> Option<WhereClause> {
    match (binrw_input.bound(), where_clause) {
        (None, where_clause) => where_clause.cloned(),
        (Some(bound), where_clause) if bound.predicates().is_empty() => where_clause.cloned(),
        (Some(bound), None) => Some(WhereClause {
            where_token: Token![where](Span::call_site()),
            predicates: bound.predicates().clone(),
        }),
        (Some(bound), Some(where_clause)) => {
            let mut where_clause = where_clause.clone();
            where_clause.predicates.extend(bound.predicates().clone());
            Some(where_clause)
        }
    }
}

fn get_args_lifetime(span: proc_macro2::Span) -> syn::Lifetime {
    syn::Lifetime::new(&format!("'{ARGS_LIFETIME}"), span)
}

fn get_assertions(assertions: &[Assert]) -> impl Iterator<Item = TokenStream> + '_ {
    assertions.iter().map(
        |Assert {
             kw_span,
             condition,
             consequent,
             ..
         }| {
            let error_fn = match &consequent {
                AssertionError::Message(message) => {
                    quote! { #ASSERT_ERROR_FN::<_, fn() -> !>::Message(|| { #message }) }
                }
                AssertionError::Error(error) => {
                    quote! { #ASSERT_ERROR_FN::Error::<fn() -> &'static str, _>(|| { #error }) }
                }
            };

            quote_spanned_any! {*kw_span=>
                #ASSERT(#condition, #POS, #error_fn)?;
            }
        },
    )
}

fn get_destructured_imports(
    imports: &Imports,
    type_name: Option<&Ident>,
    is_write: bool,
) -> Option<TokenStream> {
    match imports {
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

fn get_endian(endian: &CondEndian) -> TokenStream {
    match endian {
        CondEndian::Inherited => OPT.to_token_stream(),
        CondEndian::Fixed(endian) => endian.to_token_stream(),
        CondEndian::Cond(endian, condition) => {
            let (true_cond, false_cond) = (endian, endian.flipped());
            quote! {
                if (#condition) {
                    #true_cond
                } else {
                    #false_cond
                }
            }
        }
    }
}

fn get_map_err(pos: IdentStr, span: Span) -> TokenStream {
    quote_spanned_any! { span=>
        .map_err(|e| {
            #BIN_ERROR::Custom {
                pos: #pos,
                err: Box::new(e) as _,
            }
        })
    }
}

fn get_passed_args(field: &StructField, stream: &TokenStream) -> Option<TokenStream> {
    let args = &field.args;
    let span = args.span().unwrap_or_else(|| field.ty.span());
    match args {
        PassedArgs::Named(fields) => Some({
            let extra_args = directives_to_args(field, stream);
            quote_spanned_any! { span=>
                #ARGS_MACRO! { #extra_args #(#fields, )* }
            }
        }),
        PassedArgs::List(list) => Some(quote_spanned! {span=> (#(#list,)*) }),
        PassedArgs::Tuple(tuple) => Some(tuple.as_ref().clone()),
        PassedArgs::None => {
            let extra_args = directives_to_args(field, stream);
            (!extra_args.is_empty()).then(|| {
                quote_spanned_any! { span=> #ARGS_MACRO! { #extra_args } }
            })
        }
    }
}

fn get_try_calc(pos: IdentStr, ty: &Type, calc: &TokenStream) -> TokenStream {
    let map_err = get_map_err(pos, calc.span());
    quote_spanned! {ty.span()=> {
        let #TEMP: ::core::result::Result<#ty, _> = #calc;
        #TEMP #map_err ?
    }}
}

fn directives_to_args(field: &StructField, stream: &TokenStream) -> TokenStream {
    let args = field
        .count
        .as_ref()
        .map(|count| {
            quote_spanned_any! {count.span()=>
                count: {
                    let #TEMP = #count;
                    #[allow(clippy::useless_conversion, clippy::unnecessary_fallible_conversions)]
                    usize::try_from(#TEMP).map_err(|_| {
                        extern crate alloc;
                        #BIN_ERROR::AssertFail {
                            pos: #SEEK_TRAIT::stream_position(#stream)
                                .unwrap_or_default(),
                            // This is using debug formatting instead of display
                            // formatting to reduce the chance of some
                            // additional confusing error complaining about
                            // Display not being implemented if someone tries
                            // using a bogus type with `count`
                            message: alloc::format!("count {:?} out of range of usize", #TEMP)
                        }
                    })?
                }
            }
        })
        .into_iter()
        .chain(
            field
                .offset
                .as_ref()
                .map(|offset| quote_spanned! { offset.span()=> offset: #offset }),
        );
    quote! { #(#args,)* }
}
