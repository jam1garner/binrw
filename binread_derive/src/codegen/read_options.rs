mod debug_template;
mod r#enum;
mod r#struct;

use crate::parser::{Assert, CondEndian, Endian, Input, Map, PassedArgs};
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
            Input::UnitOnlyEnum(e) => generate_unit_enum(e),
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

// TODO: replace all functions that are only passed tla with a method on TopLevelAttrs

fn get_prelude(input: &Input) -> TokenStream {
    let arg_vars = input.imports().idents();
    let options = get_read_options_with_endian(&input.endian());
    let magic_handler = get_magic_pre_assertion(&input);

    quote! {
        let #arg_vars = #ARGS;
        let #OPT = #options;
        #magic_handler
    }
}

fn get_passed_args(args: &PassedArgs) -> TokenStream {
    match args {
        PassedArgs::List(list) => quote! { (#(#list,)*) },
        PassedArgs::Tuple(tuple) => tuple.clone(),
        PassedArgs::None => quote! { () },
    }
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
                Endian::Big => (quote!{ #ENDIAN_ENUM::Big }, quote!{ #ENDIAN_ENUM::Little }),
                Endian::Little => (quote!{ #ENDIAN_ENUM::Little }, quote!{ #ENDIAN_ENUM::Big }),
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

fn get_read_options_override_keys(options: impl Iterator<Item = (IdentStr, TokenStream)>) -> TokenStream {
    let mut set_options = options.map(|(key, value)| {
        quote! {
            #TEMP.#key = #value;
        }
    }).peekable();

    if set_options.peek().is_none() {
        quote! { #OPT }
    } else {
        quote! {
            &{
                let mut #TEMP = #OPT.clone();
                #(#set_options)*
                #TEMP
            }
        }
    }
}

fn get_read_options_with_endian(endian: &CondEndian) -> TokenStream {
    get_read_options_override_keys(get_endian_tokens(endian).into_iter())
}

fn get_magic_pre_assertion(tla: &Input) -> TokenStream {
    let handle_error = debug_template::handle_error();
    let magic = tla.magic()
        .as_ref()
        .map(|magic|{
            let (_, ref magic) = **magic;
            quote!{
                #ASSERT_MAGIC(#READER, #magic, #OPT)#handle_error?;
            }
        });
    let pre_asserts = get_assertions(&tla.pre_assert());

    quote! {
        #magic
        #(#pre_asserts)*
    }
}

fn get_assertions(asserts: &[Assert]) -> impl Iterator<Item = TokenStream> + '_ {
    asserts
        .iter()
        .map(|Assert(assert, error)| {
            let handle_error = debug_template::handle_error();
            let error = error.as_ref().map_or_else(
                || quote!{{
                    let mut x = Some(||{});
                    x = None;
                    x
                }},
                |err|{
                    quote!{Some(
                        || { #err }
                    )}
                });
            let assert_string = assert.to_string();

            quote!{
                #ASSERT(#READER, #assert, #assert_string, #error)#handle_error?;
            }
        })
}
