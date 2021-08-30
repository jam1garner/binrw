mod r#enum;
mod r#struct;

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::{Assert, AssertionError, CondEndian, Endian, read::Input, Magic, Map};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

use r#enum::{generate_data_enum, generate_unit_enum};
use r#struct::{generate_struct, generate_unit_struct};

pub(crate) fn generate(input: &Input, derive_input: &syn::DeriveInput) -> TokenStream {
    let name = Some(&derive_input.ident);
    let inner = match input.map() {
        Map::None => match input {
            Input::UnitStruct(_) => generate_unit_struct(input, name, None),
            Input::Struct(s) => generate_struct(input, name, s),
            Input::Enum(e) => generate_data_enum(input, name, e),
            Input::UnitOnlyEnum(e) => generate_unit_enum(input, name, e),
        },
        Map::Try(map) => {
            let map_err = get_map_err(POS);
            quote! {
                #READ_METHOD(#READER, #OPT, #ARGS).and_then(|value| {
                    #map(value)#map_err
                })
            }
        }
        Map::Map(map) => quote! {
            #READ_METHOD(#READER, #OPT, #ARGS).map(#map)
        },
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
        if let Some(imports) = self.input.imports().destructure(name) {
            let head = self.out;
            self.out = quote! {
                #head
                let #imports = #ARGS;
            };
        }

        self
    }

    fn add_options(mut self) -> Self {
        let options = ReadOptionsGenerator::new(OPT)
            .endian(self.input.endian())
            .finish();

        if !options.is_empty() {
            let head = self.out;
            self.out = quote! {
                #head
                #options
            };
        }

        self
    }

    fn add_magic_pre_assertion(mut self) -> Self {
        let head = self.out;
        let magic = get_magic(self.input.magic(), &OPT);
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

fn get_assertions(assertions: &[Assert]) -> impl Iterator<Item = TokenStream> + '_ {
    assertions.iter().map(
        |Assert {
             condition,
             consequent,
         }| {
            let error_fn = match &consequent {
                Some(AssertionError::Message(message)) => {
                    quote! { #ASSERT_ERROR_FN::<_, fn() -> !>::Message(|| { #message }) }
                }
                Some(AssertionError::Error(error)) => {
                    quote! { #ASSERT_ERROR_FN::Error::<fn() -> &'static str, _>(|| { #error }) }
                }
                None => {
                    let condition = condition.to_string();
                    quote! { #ASSERT_ERROR_FN::Message::<_, fn() -> !>(|| { #condition }) }
                }
            };

            quote! {
                #ASSERT(#condition, #POS, #error_fn)?;
            }
        },
    )
}

fn get_magic(magic: &Magic, options_var: &impl ToTokens) -> Option<TokenStream> {
    magic.as_ref().map(|magic| {
        let magic = magic.deref_value();
        quote! {
            #ASSERT_MAGIC(#READER, #magic, #options_var)?;
        }
    })
}

fn get_map_err(pos: IdentStr) -> TokenStream {
    quote! {
        .map_err(|e| {
            #BIN_ERROR::Custom {
                pos: #pos,
                err: Box::new(e) as _,
            }
        })
    }
}

struct ReadOptionsGenerator {
    out: TokenStream,
    options_var: TokenStream,
}

impl ReadOptionsGenerator {
    fn new(options_var: impl quote::ToTokens) -> Self {
        Self {
            out: TokenStream::new(),
            options_var: options_var.into_token_stream(),
        }
    }

    fn endian(mut self, endian: &CondEndian) -> Self {
        let endian = match endian {
            CondEndian::Inherited => return self,
            CondEndian::Fixed(Endian::Big) => quote! { #ENDIAN_ENUM::Big },
            CondEndian::Fixed(Endian::Little) => quote! { #ENDIAN_ENUM::Little },
            CondEndian::Cond(endian, condition) => {
                let (true_cond, false_cond) = match endian {
                    Endian::Big => (
                        quote! { #ENDIAN_ENUM::Big },
                        quote! { #ENDIAN_ENUM::Little },
                    ),
                    Endian::Little => (
                        quote! { #ENDIAN_ENUM::Little },
                        quote! { #ENDIAN_ENUM::Big },
                    ),
                };

                quote! {
                    if (#condition) {
                        #true_cond
                    } else {
                        #false_cond
                    }
                }
            }
        };

        let head = self.out;
        self.out = quote! {
            #head
            #TEMP.endian = #endian;
        };

        self
    }

    fn finish(self) -> TokenStream {
        let options_var = self.options_var;
        if self.out.is_empty() {
            quote! {
                let #options_var = #OPT;
            }
        } else {
            let setters = self.out;
            quote! {
                let #options_var = &{
                    let mut #TEMP = *#OPT;
                    #setters
                    #TEMP
                };
            }
        }
    }

    fn offset(mut self, offset: &Option<TokenStream>) -> Self {
        if let Some(offset) = &offset {
            let head = self.out;
            self.out = quote! {
                #head
                #TEMP.offset = #offset;
            };
        }

        self
    }
}
