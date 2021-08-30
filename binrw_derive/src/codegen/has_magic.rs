use super::sanitization::HAS_MAGIC;
use crate::parser::read::Input;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate(input: &Input, derive_input: &syn::DeriveInput) -> Option<TokenStream> {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();
    let magic = input.magic().as_ref();
    magic.map(|magic| {
        let ty: TokenStream = magic.kind().into();
        let val = magic.deref_value();
        quote! {
            impl #impl_generics #HAS_MAGIC for #name #ty_generics #where_clause {
                type MagicType = #ty;
                const MAGIC: Self::MagicType = #val;
            }
        }
    })
}
