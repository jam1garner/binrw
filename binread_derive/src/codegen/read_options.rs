mod debug_template;
mod r#enum;
mod r#struct;

use crate::parser::{Assert, CondEndian, Endian, Input, Map};
#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use r#enum::{generate_data_enum, generate_unit_enum};
use r#struct::{generate_struct, generate_unit_struct};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn generate(ident: &Ident, input: &Input) -> TokenStream {
    let inner = match input.map() {
        Map::None => match input {
            Input::UnitStruct(_) => generate_unit_struct(input),
            Input::Struct(s) => generate_struct(ident, input, s),
            Input::Enum(e) => generate_data_enum(e),
            Input::UnitOnlyEnum(e) => generate_unit_enum(input, e),
        },
        Map::Try(map) => quote! {
            #READ_METHOD(#READER, #OPT, #ARGS).and_then(#map)
        },
        Map::Map(map) => quote! {
            #READ_METHOD(#READER, #OPT, #ARGS).map(#map)
        },
    };

    quote! {
        let #POS = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))?;
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

impl <'input> PreludeGenerator<'input> {
    fn new(input: &'input Input) -> Self {
        Self {
            input,
            out: TokenStream::new()
        }
    }

    fn finish(self) -> TokenStream {
        self.out
    }

    fn add_imports(mut self) -> Self {
        if let Some(imports) = self.input.imports().idents() {
            let value = &self.out;
            self.out = quote! {
                #value
                let #imports = #ARGS;
            };
        }

        self
    }

    fn add_options(mut self) -> Self {
        let value = &self.out;
        if let Some(options) = get_read_options_override_keys(get_endian_tokens(&self.input.endian()).into_iter()) {
            self.out = quote! {
                #value
                let #OPT = #options;
            };
        }

        self
    }

    fn add_magic_pre_assertion(mut self) -> Self {
        let magic = self.input.magic().as_ref().map(|magic| {
            let handle_error = debug_template::handle_error();
            let magic = &magic.1;
            quote! {
                #ASSERT_MAGIC(#READER, #magic, #OPT)#handle_error?;
            }
        });
        let pre_asserts = get_assertions(&self.input.pre_assert());
        let value = &self.out;

        self.out = quote! {
            #value
            #magic
            #(#pre_asserts)*
        };

        self
    }
}

fn get_prelude(input: &Input) -> TokenStream {
    PreludeGenerator::new(input)
        .add_imports()
        .add_options()
        .add_magic_pre_assertion()
        .finish()
}

ident_str! {
    ENDIAN = "endian";
}

fn get_endian_tokens(endian: &CondEndian) -> Option<(IdentStr, TokenStream)> {
    match endian {
        CondEndian::Inherited => None,
        CondEndian::Fixed(Endian::Big) => Some((ENDIAN, quote! { #ENDIAN_ENUM::Big })),
        CondEndian::Fixed(Endian::Little) => Some((ENDIAN, quote! { #ENDIAN_ENUM::Little })),
        CondEndian::Cond(endian, condition) => {
            let (true_cond, false_cond) = match endian {
                Endian::Big => (quote! { #ENDIAN_ENUM::Big }, quote! { #ENDIAN_ENUM::Little }),
                Endian::Little => (quote! { #ENDIAN_ENUM::Little }, quote! { #ENDIAN_ENUM::Big }),
            };

            Some((ENDIAN, quote! {
                if (#condition) {
                    #true_cond
                } else {
                    #false_cond
                }
            }))
        }
    }
}

fn get_read_options_override_keys(options: impl Iterator<Item = (IdentStr, TokenStream)>) -> Option<TokenStream> {
    let mut set_options = options.map(|(key, value)| {
        quote! {
            #TEMP.#key = #value;
        }
    }).peekable();

    if set_options.peek().is_none() {
        None
    } else {
        Some(quote! {
            &{
                let mut #TEMP = *#OPT;
                #(#set_options)*
                #TEMP
            }
        })
    }
}

fn get_assertions(asserts: &[Assert]) -> impl Iterator<Item = TokenStream> + '_ {
    asserts.iter().map(|Assert(assert, error)| {
        let handle_error = debug_template::handle_error();
        let error = error.as_ref().map_or_else(
            || quote! { None::<fn() -> ()> },
            |error| quote! { Some(|| { #error }) }
        );
        let assert_string = assert.to_string();
        quote! {
            #ASSERT(#READER, #assert, #assert_string, #error)#handle_error?;
        }
    })
}
