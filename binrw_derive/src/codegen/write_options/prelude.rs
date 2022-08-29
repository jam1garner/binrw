use crate::{
    codegen::sanitization::{ARGS, OPT, WRITER, WRITE_METHOD},
    parser::{CondEndian, Input, Magic},
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub(crate) struct PreludeGenerator<'a> {
    out: TokenStream,
    input: Option<&'a Input>,
    name: Option<&'a Ident>,
}

impl<'a> PreludeGenerator<'a> {
    pub(crate) fn new(out: TokenStream, input: Option<&'a Input>, name: Option<&'a Ident>) -> Self {
        Self { out, input, name }
    }

    pub(crate) fn prefix_imports(mut self) -> Self {
        if let Some(imports) = self
            .input
            .and_then(|input| input.imports().destructure(self.name, true))
        {
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
            let magic = magic.match_value();
            let out = self.out;
            self.out = quote! {
                #WRITE_METHOD (
                    &#magic,
                    #WRITER,
                    &#OPT,
                    ()
                )?;

                #out
            };
        }

        self
    }

    pub(crate) fn prefix_endian(mut self, endian: &CondEndian) -> Self {
        let out = self.out;
        let set_endian = match endian {
            CondEndian::Inherited => None,
            CondEndian::Fixed(endian) => Some({
                quote! {
                    let #OPT = #OPT.clone().with_endian(#endian);
                    let #OPT = &#OPT;
                }
            }),
            CondEndian::Cond(endian, cond) => Some({
                let else_endian = endian.flipped();
                quote! {
                    let #OPT = #OPT.clone().with_endian(if #cond { #endian } else { #else_endian });
                    let #OPT = &#OPT;
                }
            }),
        };

        self.out = quote! {
            #set_endian
            #out
        };

        self
    }

    pub(crate) fn finish(self) -> TokenStream {
        self.out
    }
}
