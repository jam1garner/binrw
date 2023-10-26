use super::{prelude::PreludeGenerator, struct_field::write_field};
use crate::binrw::{
    codegen::sanitization::{THIS, WRITER},
    parser::{Input, Struct},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(super) fn generate_struct(input: &Input, name: Option<&Ident>, st: &Struct) -> TokenStream {
    StructGenerator::new(input, st, name, &input.stream_ident_or(WRITER))
        .write_fields()
        .prefix_prelude()
        .prefix_borrow_fields()
        .finish()
}

pub(super) struct StructGenerator<'input> {
    input: &'input Input,
    st: &'input Struct,
    name: Option<&'input Ident>,
    writer_var: &'input TokenStream,
    out: TokenStream,
}

impl<'input> StructGenerator<'input> {
    pub(super) fn new(
        input: &'input Input,
        st: &'input Struct,
        name: Option<&'input Ident>,
        writer_var: &'input TokenStream,
    ) -> Self {
        Self {
            input,
            st,
            name,
            writer_var,
            out: TokenStream::new(),
        }
    }

    pub(super) fn prefix_prelude(mut self) -> Self {
        self.out = PreludeGenerator::new(self.out, self.input, self.name, self.writer_var)
            .prefix_map_stream()
            .prefix_magic(&self.st.magic)
            .prefix_endian(&self.st.endian)
            .prefix_assertions()
            .prefix_imports()
            .finish();

        self
    }

    pub(super) fn write_fields(mut self) -> Self {
        let write_fields = self
            .st
            .fields
            .iter()
            .map(|field| write_field(self.writer_var, field));

        self.out = quote! {
            #(#write_fields)*
        };

        self
    }

    pub(super) fn prefix_borrow_fields(mut self) -> Self {
        let borrow_fields = self.name.map(|name| {
            let pattern = self.st.fields_pattern();

            Some(quote! {
                let #name #pattern = self;
            })
        });

        let out = self.out;
        self.out = quote! {
            let #THIS = self;
            #borrow_fields
            #out
        };

        self
    }

    pub(super) fn finish(self) -> TokenStream {
        self.out
    }
}
