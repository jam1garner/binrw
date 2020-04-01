use proc_macro2::TokenStream;
use super::TopLevelAttrs;
use crate::CompileError;
use quote::quote;

/// Generate the argument type for the derived impl
pub fn generate(tla: &TopLevelAttrs) -> Result<TokenStream, CompileError> {
    let types: Vec<_> = tla.import.types().collect();

    Ok(quote!{
        (#(#types,)*)
    })
}

