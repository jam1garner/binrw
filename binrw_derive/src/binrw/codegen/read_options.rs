mod r#enum;
mod map;
mod r#struct;

use super::{get_assertions, get_destructured_imports};
use crate::binrw::{
    codegen::{
        get_endian,
        sanitization::{ARGS, ASSERT_MAGIC, OPT, POS, READER, SEEK_FROM, SEEK_TRAIT},
    },
    parser::{Input, Magic, Map},
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use r#enum::{generate_data_enum, generate_unit_enum};
use r#struct::{generate_struct, generate_unit_struct};
use syn::Ident;

pub(crate) fn generate(input: &Input, derive_input: &syn::DeriveInput) -> TokenStream {
    let name = Some(&derive_input.ident);
    let inner = match input.map() {
        Map::None => match input {
            Input::UnitStruct(_) => generate_unit_struct(input, name, None),
            Input::Struct(s) => generate_struct(input, name, s),
            Input::Enum(e) => generate_data_enum(input, name, e),
            Input::UnitOnlyEnum(e) => generate_unit_enum(input, name, e),
        },
        Map::Try(map) => map::generate_try_map(input, name, map),
        Map::Map(map) => map::generate_map(input, name, map),
    };

    quote! {
        let #POS = #SEEK_TRAIT::stream_position(#READER)?;
        (|| {
            #inner
        })().or_else(|error| {
            #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#POS))?;
            Err(error)
        })
    }
}

struct PreludeGenerator<'input> {
    input: &'input Input,
    out: TokenStream,
}

impl<'input> PreludeGenerator<'input> {
    fn new(input: &'input Input) -> Self {
        Self {
            input,
            out: TokenStream::new(),
        }
    }

    fn finish(self) -> TokenStream {
        self.out
    }

    fn add_imports(mut self, name: Option<&Ident>) -> Self {
        if let Some(imports) = get_destructured_imports(self.input.imports(), name, false) {
            let head = self.out;
            self.out = quote! {
                #head
                let #imports = #ARGS;
            };
        }

        self
    }

    fn add_endian(mut self) -> Self {
        let endian = get_endian(self.input.endian());
        let head = self.out;
        self.out = quote! {
            #head
            let #OPT = #endian;
        };
        self
    }

    fn add_magic_pre_assertion(mut self) -> Self {
        let head = self.out;
        let magic = get_magic(self.input.magic(), OPT);
        let pre_assertions = get_assertions(self.input.pre_assertions());
        self.out = quote! {
            #head
            #magic
            #(#pre_assertions)*
        };

        self
    }

    fn reset_position_after_magic(mut self) -> Self {
        if self.input.magic().is_some() {
            let head = self.out;
            self.out = quote! {
                #head
                let #POS = #SEEK_TRAIT::stream_position(#READER)?;
            };
        };

        self
    }
}

fn get_magic(magic: &Magic, endian_var: impl ToTokens) -> Option<TokenStream> {
    magic.as_ref().map(|magic| {
        let magic = magic.deref_value();
        quote! {
            #ASSERT_MAGIC(#READER, #magic, #endian_var)?;
        }
    })
}
