use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote,
    visit_mut::{visit_type_mut, VisitMut},
    Type,
};

struct LifetimeReplacer;

impl VisitMut for LifetimeReplacer {
    fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
        if lifetime.ident == "_" {
            *lifetime = parse_quote!('arg);
        }
    }
}

pub(crate) fn args_type(mut ty: Type) -> TokenStream {
    visit_type_mut(&mut LifetimeReplacer, &mut ty);

    quote! {
        dyn for<'arg> ::binrw::ArgType<'arg, Item = #ty>
    }
}
