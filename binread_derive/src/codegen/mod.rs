mod read_options;
pub(crate) mod sanitization;

use crate::parser::TopLevelAttrs;
use proc_macro2::TokenStream;
use syn::Error;

pub fn generate(input: &syn::DeriveInput) -> syn::Result<GeneratedCode> {
    if let syn::Data::Union(ref union) = input.data {
        return Err(Error::new(union.union_token.span, "Unions are not supported"));
    }

    let tla = TopLevelAttrs::try_from_attrs(&input.attrs)?;

    Ok(GeneratedCode {
        arg_type: tla.import.types(),
        read_opt_impl: read_options::generate(input, &tla)?,
    })
}

pub struct GeneratedCode {
    pub read_opt_impl: TokenStream,
    pub arg_type: TokenStream
}
