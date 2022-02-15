use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote,
    visit_mut::{visit_type_mut, VisitMut},
    Type,
};

pub(crate) struct LifetimeReplacer(bool);

impl LifetimeReplacer {
    pub(crate) fn new() -> Self {
        Self(false)
    }

    pub(crate) fn had_lifetime(&self) -> bool {
        self.0
    }
}

impl VisitMut for LifetimeReplacer {
    fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
        if lifetime.ident == "_" {
            self.0 = true;
            *lifetime = parse_quote!('arg);
        }
    }
}

pub(crate) fn args_type(mut ty: Type) -> TokenStream {
    visit_type_mut(&mut LifetimeReplacer::new(), &mut ty);

    quote! {
        dyn for<'arg> ::binrw::ArgType<'arg, Item = #ty>
    }
}
