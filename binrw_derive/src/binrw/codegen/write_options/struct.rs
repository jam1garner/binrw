use super::{prelude::PreludeGenerator, struct_field::write_field};
use crate::binrw::{
    codegen::{get_assertions, sanitization::WRITER},
    parser::{Input, Struct},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(super) fn generate_struct(input: &Input, name: Option<&Ident>, st: &Struct) -> TokenStream {
    StructGenerator::new(Some(input), st, name, &input.stream_ident_or(WRITER))
        .write_fields()
        .prefix_assertions()
        .prefix_prelude()
        .prefix_borrow_fields()
        .finish()
}

pub(crate) struct StructGenerator<'input> {
    input: Option<&'input Input>,
    st: &'input Struct,
    name: Option<&'input Ident>,
    writer_var: &'input TokenStream,
    out: TokenStream,
}

impl<'input> StructGenerator<'input> {
    pub(crate) fn new(
        input: Option<&'input Input>,
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

    pub(crate) fn prefix_prelude(mut self) -> Self {
        self.out = PreludeGenerator::new(self.out, self.input, self.name, self.writer_var)
            .prefix_map_stream()
            .prefix_magic(&self.st.magic)
            .prefix_endian(&self.st.endian)
            .prefix_imports()
            .finish();

        self
    }

    fn prefix_assertions(mut self) -> Self {
        let assertions = get_assertions(&self.st.assertions);

        let out = self.out;
        self.out = quote! {
            #(#assertions)*
            #out
        };

        self
    }

    pub(crate) fn write_fields(mut self) -> Self {
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
