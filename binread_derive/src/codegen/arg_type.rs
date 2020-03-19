use proc_macro2::TokenStream;
use super::TopLevelAttrs;
use crate::CompileError;
use quote::quote;

pub fn generate(tla: &TopLevelAttrs) -> Result<TokenStream, CompileError> {
    let types: Vec<_> = tla.import.types().collect();

    Ok(quote!{
        (#(#types,)*)
    })
}

