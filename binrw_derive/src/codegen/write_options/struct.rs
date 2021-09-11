use crate::parser::write::{Input, Struct};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

//#[allow(clippy::wildcard_imports)]
//use crate::codegen::sanitization::*;

use super::prelude::PreludeGenerator;
use super::struct_field::write_field;

pub(super) fn generate_struct(input: &Input, name: Option<&Ident>, st: &Struct) -> TokenStream {
    StructGenerator::new(Some(input), st, name)
        .write_fields()
        .prefix_prelude()
        .prefix_borrow_fields()
        .finish()
}

pub(crate) struct StructGenerator<'input> {
    input: Option<&'input Input>,
    st: &'input Struct,
    name: Option<&'input Ident>,
    out: TokenStream,
}

impl<'input> StructGenerator<'input> {
    pub(crate) fn new(
        input: Option<&'input Input>,
        st: &'input Struct,
        name: Option<&'input Ident>,
    ) -> Self {
        Self {
            input,
            st,
            name,
            out: TokenStream::new(),
        }
    }

    pub(crate) fn prefix_prelude(mut self) -> Self {
        self.out = PreludeGenerator::new(self.out, self.input, self.name)
            .prefix_magic(&self.st.magic)
            .prefix_endian(&self.st.endian)
            .prefix_imports()
            .finish();

        self
    }

    pub(crate) fn write_fields(mut self) -> Self {
        let write_fields = self.st.fields.iter().map(write_field);

        self.out = quote! {
            #(#write_fields)*
        };

        self
    }

    pub(crate) fn prefix_borrow_fields(mut self) -> Self {
        let borrow_fields = self.name.as_ref().map(|name| {
            let pattern = self.st.fields_pattern();

            Some(quote! {
                let #name #pattern = self;
            })
        });

        let out = self.out;
        self.out = quote! {
            #borrow_fields
            #out
        };

        self
    }

    pub(crate) fn finish(self) -> TokenStream {
        self.out
    }
}
