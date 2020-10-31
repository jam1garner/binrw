use proc_macro2::TokenStream;
use super::TopLevelAttrs;
use crate::CompileError;
use syn::DeriveInput;
use quote::quote;

pub fn generate(_: &DeriveInput, _: &TopLevelAttrs) -> Result<TokenStream, CompileError> {
    Ok(quote!{Ok(())})
}
