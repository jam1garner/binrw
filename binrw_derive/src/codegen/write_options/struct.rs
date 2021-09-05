use crate::parser::write::{Input, Struct, StructField};
use crate::parser::CondEndian;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;

pub(super) fn generate_struct(input: &Input, _name: Option<&Ident>, st: &Struct) -> TokenStream {
    StructGenerator::new(input, st).write_fields().finish()
}

struct StructGenerator<'input> {
    #[allow(dead_code)]
    input: &'input Input,
    st: &'input Struct,
    out: TokenStream,
}

impl<'input> StructGenerator<'input> {
    fn new(input: &'input Input, st: &'input Struct) -> Self {
        Self {
            input,
            st,
            out: TokenStream::new(),
        }
    }

    fn write_fields(mut self) -> Self {
        let write_fields = self.st.fields.iter().map(write_field);

        self.out = quote! {
            #(#write_fields)*
        };

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }
}

fn write_field(field: &StructField) -> TokenStream {
    StructFieldGenerator::new(field)
        .write_field()
        .prefix_args()
        .finish()
}

struct StructFieldGenerator<'input> {
    field: &'input StructField,
    out: TokenStream,
}

impl<'a> StructFieldGenerator<'a> {
    fn new(field: &'a StructField) -> Self {
        Self {
            field,
            out: TokenStream::new(),
        }
    }

    fn args_ident(&self) -> Ident {
        make_ident(&self.field.ident, "args")
    }

    fn specify_endian(&self) -> Option<TokenStream> {
        match &self.field.endian {
            CondEndian::Inherited => None,
            CondEndian::Fixed(endian) => Some({
                let endian = endian.as_binrw_endian();
                quote! {
                    .clone().with_endian(#endian)
                }
            }),
            CondEndian::Cond(endian, cond) => Some({
                let else_endian = endian.flipped().as_binrw_endian();
                let endian = endian.as_binrw_endian();
                quote! {
                    .clone().with_endian(if #cond { #endian } else { #else_endian })
                }
            }),
        }
    }

    fn write_field(mut self) -> Self {
        let name = &self.field.ident;
        let args = self.args_ident();
        let specify_endian = self.specify_endian();

        self.out = quote! {
            #WRITE_METHOD (
                &self.#name,
                #WRITER,
                &#OPT#specify_endian,
                #args
            )?;
        };

        self
    }

    fn prefix_args(mut self) -> Self {
        let args = self.args_ident();

        let out = &self.out;
        self.out = quote! {
            // TODO: add support for passing arguments
            let #args = ();
            #out
        };

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }
}
