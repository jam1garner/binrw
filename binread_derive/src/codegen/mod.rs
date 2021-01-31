pub(crate) mod read_options;
pub(crate) mod sanitization;

use crate::parser::TopLevelAttrs;
use proc_macro2::TokenStream;

pub fn generate(input: &syn::DeriveInput) -> syn::Result<GeneratedCode> {
    let tla = TopLevelAttrs::try_from_input(&input)?;

    Ok(GeneratedCode {
        arg_type: tla.import.types(),
        read_opt_impl: read_options::generate(input, &tla)?,
    })
}

pub struct GeneratedCode {
    pub read_opt_impl: TokenStream,
    pub arg_type: TokenStream
}
