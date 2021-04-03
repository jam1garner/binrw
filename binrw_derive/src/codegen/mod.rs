#[macro_use]
pub(crate) mod sanitization;
mod read_options;
pub(crate) mod typed_builder;

mod imports;

use crate::parser::{Input, ParseResult};
use proc_macro2::TokenStream;
use quote::quote;
#[allow(clippy::wildcard_imports)]
use sanitization::*;

pub(crate) fn generate_impl(
    derive_input: &syn::DeriveInput,
    binread_input: &ParseResult<Input>,
) -> TokenStream {
    // If there is a parsing error, a BinRead impl still needs to be
    // generated to avoid misleading errors at all call sites that use the
    // BinRead trait
    let ((arg_type, arg_type_declaration), read_opt_impl) = match binread_input {
        ParseResult::Ok(binread_input) => (
            binread_input.imports().args_type(&derive_input.ident),
            read_options::generate(&binread_input, derive_input),
        ),
        ParseResult::Partial(binread_input, error) => (
            binread_input.imports().args_type(&derive_input.ident),
            error.to_compile_error(),
        ),
        ParseResult::Err(error) => ((quote! { () }, None), error.to_compile_error()),
    };

    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();
    quote! {
        #[allow(non_snake_case)]
        impl #impl_generics #TRAIT_NAME for #name #ty_generics #where_clause {
            type Args = #arg_type;

            fn read_options<R: #READ_TRAIT + #SEEK_TRAIT>
                (#READER: &mut R, #OPT: &#OPTIONS, #ARGS: Self::Args)
                -> #BIN_RESULT<Self>
            {
                #read_opt_impl
            }
        }

        #arg_type_declaration
    }
}
