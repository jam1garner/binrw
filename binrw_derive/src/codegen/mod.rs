mod imports;
mod meta;
pub(crate) mod named_args;
mod read_options;
pub(crate) mod sanitization;
mod write_options;

use crate::{
    parser::{Assert, AssertionError, Input, ParseResult, PassedArgs, StructField},
    util::quote_spanned_any,
};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use sanitization::{
    ARGS, ARGS_MACRO, ASSERT, ASSERT_ERROR_FN, BINREAD_TRAIT, BINWRITE_TRAIT, BIN_RESULT, OPT, POS,
    READER, READ_OPTIONS, READ_TRAIT, SEEK_TRAIT, WRITER, WRITE_OPTIONS, WRITE_TRAIT,
};
use syn::{spanned::Spanned, DeriveInput};

pub(crate) fn generate_impl<const WRITE: bool>(
    derive_input: &DeriveInput,
    binrw_input: &ParseResult<Input>,
) -> TokenStream {
    let (arg_type, arg_type_declaration) = match binrw_input {
        ParseResult::Ok(binrw_input) | ParseResult::Partial(binrw_input, _) => imports::args_type(
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
                    #OPT: &#WRITE_OPTIONS,
                    #ARGS: Self::Args
                ) -> #BIN_RESULT<()>
            },
        )
    } else {
        (
            BINREAD_TRAIT,
            quote! {
                fn read_options<R: #READ_TRAIT + #SEEK_TRAIT>
                    (#READER: &mut R, #OPT: &#READ_OPTIONS, #ARGS: Self::Args)
                    -> #BIN_RESULT<Self>
            },
        )
    };

    let fn_impl = match binrw_input {
        ParseResult::Ok(binrw_input) => {
            if WRITE {
                write_options::generate(binrw_input, derive_input)
            } else {
                read_options::generate(binrw_input, derive_input)
            }
        }
        // If there is a parsing error, an impl for the trait still needs to be
        // generated to avoid misleading errors at all call sites that use the
        // trait, so emit the trait and just stick the errors inside the generated
        // function
        ParseResult::Partial(_, error) | ParseResult::Err(error) => error.to_compile_error(),
    };

    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    quote! {
        #[automatically_derived]
        #[allow(non_snake_case)]
        #[allow(clippy::redundant_closure_call)]
        impl #impl_generics #trait_name for #name #ty_generics #where_clause {
            type Args = #arg_type;

            #fn_sig {
                #fn_impl
            }
        }
    }
}

fn get_assertions(assertions: &[Assert]) -> impl Iterator<Item = TokenStream> + '_ {
    assertions.iter().map(
        |Assert {
             condition,
             consequent,
         }| {
            let error_fn = match &consequent {
                Some(AssertionError::Message(message)) => {
                    quote! { #ASSERT_ERROR_FN::<_, fn() -> !>::Message(|| { #message }) }
                }
                Some(AssertionError::Error(error)) => {
                    quote! { #ASSERT_ERROR_FN::Error::<fn() -> &'static str, _>(|| { #error }) }
                }
                None => {
                    let condition = condition.to_string();
                    quote! { #ASSERT_ERROR_FN::Message::<_, fn() -> !>(|| {
                        extern crate alloc;
                        alloc::format!("assertion failed: `{}`", #condition)
                    }) }
                }
            };

            quote! {
                #ASSERT(#condition, #POS, #error_fn)?;
            }
        },
    )
}

fn get_passed_args(field: &StructField) -> Option<TokenStream> {
    let args = &field.args;
    let span = args.span().unwrap_or_else(|| field.ty.span());
    match args {
        PassedArgs::Named(fields) => Some({
            let extra_args = directives_to_args(field);
            quote_spanned_any! { span=>
                #ARGS_MACRO! { #extra_args #(#fields, )* }
            }
        }),
        PassedArgs::List(list) => Some(quote_spanned! {span=> (#(#list,)*) }),
        PassedArgs::Tuple(tuple) => Some(tuple.as_ref().clone()),
        PassedArgs::None => {
            let extra_args = directives_to_args(field);
            (!extra_args.is_empty()).then(|| {
                quote_spanned_any! { span=> #ARGS_MACRO! { #extra_args } }
            })
        }
    }
}

fn directives_to_args(field: &StructField) -> TokenStream {
    let args = field
        .count
        .as_ref()
        .map(|count| quote! { count: ((#count) as usize) })
        .into_iter()
        .chain(
            field
                .offset
                .as_ref()
                .map(|offset| quote! { offset: (#offset) })
                .into_iter(),
        );
    quote! { #(#args,)* }
}
