#[macro_use]
pub(crate) mod sanitization;
mod has_magic;
mod imports;
mod read_options;
pub(crate) mod typed_builder;
mod types;
mod write_options;

use crate::parser::{read, write, ParseResult};
use proc_macro2::TokenStream;
use quote::quote;
#[allow(clippy::wildcard_imports)]
use sanitization::*;

pub(crate) fn generate_binread_impl(
    derive_input: &syn::DeriveInput,
    binread_input: &ParseResult<read::Input>,
) -> TokenStream {
    // Generate the argument type name and (if needed) definition
    let (arg_type, arg_type_declaration) = match binread_input {
        ParseResult::Ok(binread_input) | ParseResult::Partial(binread_input, _) => binread_input
            .imports()
            .args_type(&derive_input.ident, &derive_input.vis),
        ParseResult::Err(_) => (quote! { () }, None),
    };

    // If there is a parsing error, a BinRead impl still needs to be
    // generated to avoid misleading errors at all call sites that use the
    // BinRead trait
    let read_opt_impl = match binread_input {
        ParseResult::Ok(binread_input) => read_options::generate(binread_input, derive_input),
        ParseResult::Partial(_, error) | ParseResult::Err(error) => error.to_compile_error(),
    };

    let has_magic_impl = match binread_input {
        ParseResult::Ok(binread_input) => has_magic::generate(binread_input, derive_input),
        ParseResult::Partial(_, error) | ParseResult::Err(error) => Some(error.to_compile_error()),
    };

    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    quote! {
        #[allow(non_snake_case)]
        impl #impl_generics #BINREAD_TRAIT for #name #ty_generics #where_clause {
            type Args = #arg_type;

            fn read_options<R: #READ_TRAIT + #SEEK_TRAIT>
                (#READER: &mut R, #OPT: &#READ_OPTIONS, #ARGS: Self::Args)
                -> #BIN_RESULT<Self>
            {
                #read_opt_impl
            }
        }

        #has_magic_impl

        #arg_type_declaration
    }
}

pub(crate) fn generate_binwrite_impl(
    derive_input: &syn::DeriveInput,
    binwrite_input: &ParseResult<write::Input>,
) -> TokenStream {
    // Generate the argument type name and (if needed) definition
    let (arg_type, arg_type_declaration) = match binwrite_input {
        ParseResult::Ok(binwrite_input) | ParseResult::Partial(binwrite_input, _) => binwrite_input
            .imports()
            .args_type(&derive_input.ident, &derive_input.vis),
        ParseResult::Err(_) => (quote! { () }, None),
    };

    // If there is a parsing error, a BinWrite impl still needs to be
    // generated to avoid misleading errors at all call sites that use the
    // BinWrite trait
    let write_opt_impl = match binwrite_input {
        ParseResult::Ok(binwrite_input) => write_options::generate(binwrite_input, derive_input),
        ParseResult::Partial(_, error) | ParseResult::Err(error) => error.to_compile_error(),
    };

    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    quote! {
        impl #impl_generics #BINWRITE_TRAIT for #name #ty_generics #where_clause {
            type Args = #arg_type;

            fn write_options<W: #WRITE_TRAIT + #SEEK_TRAIT>(
                &self,
                #WRITER: &mut W,
                #OPT: &#WRITE_OPTIONS,
                #ARGS: Self::Args
            ) -> #BIN_RESULT<()> {
                #write_opt_impl
            }
        }

        #arg_type_declaration
    }
}
