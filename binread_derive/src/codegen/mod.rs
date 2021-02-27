#[macro_use]
pub(crate) mod sanitization;
mod read_options;

use crate::parser::Input;
use proc_macro2::TokenStream;
use quote::quote;
#[allow(clippy::wildcard_imports)]
use sanitization::*;

pub(crate) fn generate_impl(input: &syn::DeriveInput) -> TokenStream {
    let (arg_type, read_opt_impl) = match Input::from_input(input) {
        Ok(binread_input) => (
            binread_input.imports().types(),
            read_options::generate(&input.ident, &binread_input),
        ),
        // If there is a parsing error, a BinRead impl still needs to be
        // generated to avoid misleading errors at all call sites that use the
        // BinRead trait
        Err(error) => (quote! { () }, error.into_compile_error()),
    };

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
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
    }
}
