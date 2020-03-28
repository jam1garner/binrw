pub(crate) mod sanitization;
mod read_options;
mod after_parse;
mod arg_type;

use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use crate::{
    meta_attrs::TopLevelAttrs,
    compiler_error::{CompileError, SpanError}
};

pub fn generate(input: &syn::DeriveInput) -> Result<GeneratedCode, CompileError> {
    if let syn::Data::Union(ref union) = input.data {
        SpanError::err(union.union_token.span, "Unions not supported")?
    }

    let tla = TopLevelAttrs::from_derive_input(input)?.finalize()?;

    Ok(GeneratedCode {
        arg_type: arg_type::generate(&tla)?,
        read_opt_impl: read_options::generate(input, &tla)?,
        after_parse_impl: after_parse::generate(input, &tla)?,
    })
}

pub struct GeneratedCode {
    pub read_opt_impl: TokenStream,
    pub after_parse_impl: TokenStream,
    pub arg_type: TokenStream
}

impl GeneratedCode {
    pub fn new<T1, T2, T3>(read_opt_impl: T1, after_parse_impl: T2, arg_type: T3) -> Self
        where T1: Into<TokenStream>,
              T2: Into<TokenStream>,
              T3: Into<TokenStream>
    {
        Self {
            read_opt_impl: read_opt_impl.into(),
            after_parse_impl: after_parse_impl.into(),
            arg_type: arg_type.into()
        }
    }
}
