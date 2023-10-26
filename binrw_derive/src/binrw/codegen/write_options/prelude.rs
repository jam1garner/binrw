use crate::{
    binrw::{
        codegen::{
            get_assertions, get_destructured_imports, get_endian,
            sanitization::{ARGS, MAP_WRITER_TYPE_HINT, OPT, WRITER, WRITE_METHOD},
        },
        parser::{CondEndian, Input, Magic},
    },
    util::quote_spanned_any,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

pub(crate) struct PreludeGenerator<'a> {
    out: TokenStream,
    input: &'a Input,
    name: Option<&'a Ident>,
    writer_var: &'a TokenStream,
}

impl<'a> PreludeGenerator<'a> {
    pub(crate) fn new(
        out: TokenStream,
        input: &'a Input,
        name: Option<&'a Ident>,
        writer_var: &'a TokenStream,
    ) -> Self {
        Self {
            out,
            input,
            name,
            writer_var,
        }
    }

    pub(super) fn prefix_assertions(mut self) -> Self {
        let assertions = get_assertions(self.input.assertions());
        let out = self.out;
        self.out = quote! {
            #(#assertions)*
            #out
        };

        self
    }

    pub(crate) fn prefix_imports(mut self) -> Self {
        if let Some(imports) = get_destructured_imports(self.input.imports(), self.name, true) {
            let out = self.out;
            self.out = quote! {
                let #imports = #ARGS;
                #out
            };
        }

        self
    }

    pub(crate) fn prefix_magic(mut self, magic: &Magic) -> Self {
        if let Some(magic) = magic {
            let writer_var = &self.writer_var;
            let magic = magic.match_value();
            let out = self.out;
            self.out = quote! {
                #WRITE_METHOD (
                    &#magic,
                    #writer_var,
                    #OPT,
                    ()
                )?;

                #out
            };
        }

        self
    }

    pub(crate) fn prefix_endian(mut self, endian: &CondEndian) -> Self {
        let endian = get_endian(endian);
        let out = self.out;
        self.out = quote! {
            let #OPT = #endian;
            #out
        };

        self
    }

    pub(crate) fn prefix_map_stream(mut self) -> Self {
        if let Some(map_stream) = self.input.map_stream() {
            let outer_writer = self.input.stream_ident_or(WRITER);
            let inner_writer = &self.writer_var;
            let tail = self.out;
            self.out = quote_spanned_any! { map_stream.span()=>
                let #inner_writer = &mut #MAP_WRITER_TYPE_HINT::<W, _, _>(#map_stream)(#outer_writer);
                #tail
            };
        }

        self
    }

    pub(crate) fn finish(self) -> TokenStream {
        self.out
    }
}
