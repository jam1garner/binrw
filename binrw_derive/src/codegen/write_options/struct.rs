use crate::parser::write::{Input, Struct};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

//#[allow(clippy::wildcard_imports)]
//use crate::codegen::sanitization::*;

use super::prelude::PreludeGenerator;
use super::struct_field::write_field;

pub(super) fn generate_struct(input: &Input, name: Option<&Ident>, st: &Struct) -> TokenStream {
    StructGenerator::new(input, st, name)
        .write_fields()
        .prefix_prelude()
        .prefix_borrow_fields()
        .finish()
}

struct StructGenerator<'input> {
    #[allow(dead_code)]
    input: &'input Input,
    st: &'input Struct,
    name: Option<&'input Ident>,
    out: TokenStream,
}

impl<'input> StructGenerator<'input> {
    fn new(input: &'input Input, st: &'input Struct, name: Option<&'input Ident>) -> Self {
        Self {
            input,
            st,
            name,
            out: TokenStream::new(),
        }
    }

    fn prefix_prelude(mut self) -> Self {
        self.out = PreludeGenerator::new(self.out, self.input, self.name)
            .prefix_magic(&self.st.magic)
            .prefix_endian(&self.st.endian)
            .prefix_imports()
            .finish();

        self
    }

    fn write_fields(mut self) -> Self {
        let write_fields = self.st.fields.iter().map(write_field);

        self.out = quote! {
            #(#write_fields)*
        };

        self
    }

    fn prefix_borrow_fields(mut self) -> Self {
        let borrow_fields = self.name.as_ref().map(|name| {
            let pattern = match &self.input {
                Input::Struct(input) => {
                    let fields = self.st.iter_permanent_idents();

                    if input.is_tuple() {
                        quote! {
                            #name (#(ref #fields),*)
                        }
                    } else {
                        quote! {
                            #name { #(ref #fields),* }
                        }
                    }
                }
                Input::UnitStruct(_) => quote! { _ },
                Input::Enum(_) | Input::UnitOnlyEnum(_) => unreachable!(),
            };

            Some(quote! {
                let #pattern = self;
            })
        });

        let out = self.out;
        self.out = quote! {
            #borrow_fields
            #out
        };

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }
}
