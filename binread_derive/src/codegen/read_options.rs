mod debug_template;
mod r#enum;
mod r#struct;

use crate::parser::{Assert, AssertionError, CondEndian, Endian, Input, Magic, Map};
#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use r#enum::{generate_data_enum, generate_unit_enum};
use r#struct::{generate_struct, generate_unit_struct};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Ident;

pub(crate) fn generate(ident: &Ident, input: &Input) -> TokenStream {
    let inner = match input.map() {
        Map::None => match input {
            Input::UnitStruct(_) => generate_unit_struct(input, None),
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
            .endian(&self.input.endian())
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
        let magic = get_magic(self.input.magic(), &OPT);
        let pre_assertions = get_assertions(&self.input.pre_assertions());
        let head = self.out;

        self.out = quote! {
            #head
            #magic
            #(#pre_assertions)*
        };

        self
    }
}

fn get_assertions(assertions: &[Assert]) -> impl Iterator<Item = TokenStream> + '_ {
    assertions.iter().map(|Assert { condition, consequent }| {
        let handle_error = debug_template::handle_error();

        let error_fn = match &consequent {
            Some(AssertionError::Message(message)) =>
                quote! { #ASSERT_ERROR_FN::<_, fn() -> ()>::Message(|| { #message }) },
            Some(AssertionError::Error(error)) =>
                quote! { #ASSERT_ERROR_FN::Error::<fn() -> &'static str, _>(|| { #error }) },
            None => {
                let condition = condition.to_string();
                quote! { #ASSERT_ERROR_FN::Message::<_, fn() -> ()>(|| { #condition }) }
            },
        };

        quote! {
            #ASSERT(#condition, #POS, #error_fn)#handle_error?;
        }
    })
}

fn get_magic(magic: &Magic, options_var: &impl ToTokens) -> Option<TokenStream> {
    magic.as_ref().map(|magic| {
        let handle_error = debug_template::handle_error();
        let magic = magic.deref_value();
        quote! {
            #ASSERT_MAGIC(#READER, #magic, #options_var)#handle_error?;
        }
    })
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

    fn count(mut self, count: &Option<TokenStream>) -> Self {
        if let Some(count) = &count {
            let head = self.out;
            self.out = quote! {
                #head
                #TEMP.count = Some((#count) as usize);
            };
        }

        self
    }

    fn endian(mut self, endian: &CondEndian) -> Self {
        let endian = match endian {
            CondEndian::Inherited => return self,
            CondEndian::Fixed(Endian::Big) => quote! { #ENDIAN_ENUM::Big },
            CondEndian::Fixed(Endian::Little) => quote! { #ENDIAN_ENUM::Little },
            CondEndian::Cond(endian, condition) => {
                let (true_cond, false_cond) = match endian {
                    Endian::Big => (quote! { #ENDIAN_ENUM::Big }, quote! { #ENDIAN_ENUM::Little }),
                    Endian::Little => (quote! { #ENDIAN_ENUM::Little }, quote! { #ENDIAN_ENUM::Big }),
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

    fn variable_name(mut self, ident: &Ident) -> Self {
        if cfg!(feature = "debug_template") {
            let ident = ident.to_string();
            let head = self.out;
            self.out = quote! {
                #head
                #TEMP.variable_name = Some(#ident);
            };
        }

        self
    }
}
