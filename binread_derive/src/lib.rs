#![allow(unused_imports, unused_macros)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned, format_ident, ToTokens};
use syn::{
    spanned::Spanned,
    parse_macro_input,
    DeriveInput
};

mod codegen;
mod sanitization;
mod binread_endian;
#[macro_use] mod compiler_error;

use sanitization::*;
use proc_macro2::{TokenStream as TokenStream2, Span};
use compiler_error::{CompileError, SpanError};

fn generate_derive(input: DeriveInput, code: codegen::GeneratedCode) -> TokenStream {
    let codegen::GeneratedCode {
        read_opt_impl, after_parse_impl, arg_type
    } = code;

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote!(
        #[allow(warnings)]
        impl #impl_generics #TRAIT_NAME for #name #ty_generics #where_clause {
            type Args = #arg_type;

            fn read_options<R: #READ_TRAIT + #SEEK_TRAIT>
                (#READER: &mut R, #OPT: &#OPTIONS, #ARGS: Self::Args)
                -> #BIN_RESULT<Self>
            {
                #read_opt_impl
            }

            fn after_parse<R: #READ_TRAIT + #SEEK_TRAIT> (&mut self, #READER: &mut R,
                #OPT : &#OPTIONS, #ARGS : Self::Args, #AFTER_OPTS : &#AFTER_PARSE_OPTIONS) 
                -> #BIN_RESULT<()>
            {
                #after_parse_impl
            }
        }
    ).into()
}

#[proc_macro_derive(BinRead, attributes(binread, br))]
pub fn derive_binread(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    match codegen::generate(&input) {
        Ok(code) => {
            generate_derive(input, code)
        }
        Err(err) => {
            let error = match err {
                CompileError::SpanError(span_err) => {
                    let SpanError (span, error) = span_err;
                    let error: &str = &error;
                    quote_spanned!{ span =>
                        compile_error!(#error)
                    }
                }
                CompileError::Darling(darling_err) => {
                    darling_err.write_errors()
                }
            };
            generate_derive(input, codegen::GeneratedCode::new(
                quote!(todo!()),
                error,
                quote!(())
            ))
        }
    }
}
