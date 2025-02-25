use super::{prelude::PreludeGenerator, struct_field::write_field};
use crate::binrw::{
    codegen::sanitization::{SEEK_TRAIT, STRUCT_POS, THIS, WRITER, WRITE_ZEROES},
    parser::{Input, Struct},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(super) fn generate_struct(input: &Input, name: Option<&Ident>, st: &Struct) -> TokenStream {
    StructGenerator::new(input, st, name, &input.stream_ident_or(WRITER))
        .write_fields()
        .wrap_pad_size()
        .prefix_prelude()
        .prefix_borrow_fields()
        .prefix_imports()
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

    pub(super) fn prefix_imports(mut self) -> Self {
        self.out = PreludeGenerator::new(self.out, self.input, self.name, self.writer_var)
            .prefix_imports()
            .finish();

        self
    }

    pub(super) fn prefix_prelude(mut self) -> Self {
        self.out = PreludeGenerator::new(self.out, self.input, self.name, self.writer_var)
            .prefix_map_stream()
            .prefix_magic(&self.st.magic)
            .prefix_endian(&self.st.endian)
            .prefix_assertions()
            .finish();

        self
    }

    fn wrap_pad_size(mut self) -> Self {
        if let Some(size) = &self.st.pad_size_to {
            let writer_var = self.writer_var;
            let out = self.out;
            self.out = quote! {
                let #STRUCT_POS = #SEEK_TRAIT::stream_position(#writer_var)?;
                #out
                {
                    let pad_to_size = (#size) as u64;
                    let after_pos = #SEEK_TRAIT::stream_position(#writer_var)?;
                    if let Some(size) = after_pos.checked_sub(#STRUCT_POS) {
                        if let Some(padding) = pad_to_size.checked_sub(size) {
                            #WRITE_ZEROES(#writer_var, padding)?;
                        }
                    }
                }
            };
        }

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
