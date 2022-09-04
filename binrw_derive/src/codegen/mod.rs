mod imports;
mod meta;
mod read_options;
pub(crate) mod sanitization;
pub(crate) mod typed_builder;
mod write_options;

use crate::{
    codegen::sanitization::{ARGS_MACRO, ASSERT, ASSERT_ERROR_FN, POS},
    parser::{Assert, AssertionError, Input, ParseResult, PassedArgs, StructField},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use sanitization::{
    ARGS, BINREAD_TRAIT, BINWRITE_TRAIT, BIN_RESULT, OPT, READER, READ_OPTIONS, READ_TRAIT,
    SEEK_TRAIT, WRITER, WRITE_OPTIONS, WRITE_TRAIT,
};
use syn::{spanned::Spanned, DeriveInput};

pub(crate) fn generate_impl<const WRITE: bool>(
    derive_input: &DeriveInput,
    binrw_input: &ParseResult<Input>,
) -> TokenStream {
    let (arg_type, arg_type_declaration) = match binrw_input {
        ParseResult::Ok(binrw_input) | ParseResult::Partial(binrw_input, _) => binrw_input
            .imports()
            .args_type(&derive_input.ident, &derive_input.vis, WRITE),
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
        #[allow(non_snake_case)]
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
                    quote! { #ASSERT_ERROR_FN::Message::<_, fn() -> !>(|| { #condition }) }
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
    match args {
        PassedArgs::Named(fields) => Some(if let Some(count) = &field.count {
            quote! {
                #ARGS_MACRO! { count: ((#count) as usize) #(, #fields)* }
            }
        } else {
            quote! {
                #ARGS_MACRO! { #(#fields),* }
            }
        }),
        PassedArgs::List(list) => Some(quote! { (#(#list,)*) }),
        PassedArgs::Tuple(tuple) => {
            let tuple = tuple.as_ref();
            Some(quote! { #tuple })
        }
        PassedArgs::None => field
            .count
            .as_ref()
            .map(|count| quote! { #ARGS_MACRO! { count: ((#count) as usize) }}),
    }
    .map(|ts| fixup_span(ts, args.span().unwrap_or_else(|| field.ty.span())))
}

// For an unknown reason, this seems to be the least invasive way to associate
// the arguments correctly with the args token; quote_spanned does not get it
// done and neither does only resetting the span on only the generated tokens
fn fixup_span(ts: TokenStream, span: Span) -> TokenStream {
    ts.into_iter()
        .map(|mut tt| {
            tt.set_span(span);
            tt
        })
        .collect::<TokenStream>()
}
