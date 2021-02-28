#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::{Enum, EnumErrorMode, EnumVariant, Input, UnitEnumField, UnitOnlyEnum};
use proc_macro2::TokenStream;
use quote::quote;
use super::{
    get_assertions,
    get_read_options_with_endian,
};

pub(super) fn generate_unit_enum(en: &UnitOnlyEnum) -> TokenStream {
    let options = get_read_options_with_endian(&en.endian);

    match &en.repr {
        Some(repr) => generate_unit_enum_repr(&options, repr, &en.fields),
        None => generate_unit_enum_magic(&options, &en.fields),
    }
}

fn generate_unit_enum_repr(options: &TokenStream, repr: &TokenStream, variants: &[UnitEnumField]) -> TokenStream {
    let clauses = variants.iter().map(|variant| {
        let ident = &variant.ident;
        quote! {
            if #TEMP == Self::#ident as #repr {
                Ok(Self::#ident)
            }
        }
    });

    quote! {
        let #OPT = #options;
        let #TEMP: #repr = #READ_METHOD(#READER, #OPT, ())?;
        #(#clauses else)* {
            Err(#BIN_ERROR::NoVariantMatch {
                pos: #POS as _,
            })
        }
    }
}

fn generate_unit_enum_magic(options: &TokenStream, variants: &[UnitEnumField]) -> TokenStream {
    // TODO: The original code here looked as if it wanted to only handle magic
    // on variants without a pre-assert condition, but this just ended up
    // failing the generation with an early return whenever there was any
    // pre-assert condition. So not sure what is the desired behaviour here.

    let matches = variants.iter().filter_map(|field| {
        if let Some(magic) = &field.magic {
            let ident = &field.ident;
            let magic = &magic.1;
            Some(quote! { #magic => Ok(Self::#ident) })
        } else {
            None
        }
    });

    quote! {
        let #OPT = #options;
        match #READ_METHOD(#READER, #OPT, ())? {
            #(#matches,)*
            _ => {
                Err(#BIN_ERROR::NoVariantMatch {
                    pos: #POS as _
                })
            }
        }
    }
}

pub(super) fn generate_data_enum(en: &Enum) -> TokenStream {
    let return_all_errors = en.error_mode != EnumErrorMode::ReturnUnexpectedError;

    let (create_error_basket, return_error) = if return_all_errors {(
        quote! {
            extern crate alloc;
            let mut #ERROR_BASKET: alloc::vec::Vec<(&'static str, #BIN_ERROR)> = alloc::vec::Vec::new();
        },
        quote! {
            Err(#BIN_ERROR::EnumErrors {
                pos: #POS as _,
                variant_errors: #ERROR_BASKET
            })
        }
    )} else {(
        TokenStream::new(),
        quote! {
            Err(#BIN_ERROR::NoVariantMatch {
                pos: #POS as _
            })
        }
    )};

    let try_each_variant = en.variants
        .iter()
        .map(|variant| {
            let body = generate_variant_impl(en, variant);

            let handle_error = if return_all_errors {
                let name = variant.ident().to_string();
                quote! {
                    #ERROR_BASKET.push((#name, #TEMP.err().unwrap()));
                }
            } else {
                TokenStream::new()
            };

            quote! {
                let #TEMP = (|| {
                    #body
                })();

                if #TEMP.is_ok() {
                    return #TEMP;
                } else {
                    #handle_error
                    #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#POS))?;
                }
            }
        });

    quote! {
        #create_error_basket
        #(#try_each_variant)*
        #return_error
    }
}

// TODO: This is distressingly close to generate_struct
fn generate_variant_impl(en: &Enum, variant: &EnumVariant) -> TokenStream {
    let (fields, assertions, return_value);
    match variant {
        EnumVariant::Variant { ident, options: ds } => {
            fields = &ds.fields[..];
            assertions = {
                let assertions = get_assertions(&en.assert).chain(get_assertions(&ds.assert));
                quote! { #(#assertions)* }
            };
            // TODO: Unit kind would be here
            let out_names = ds.iter_permanent_idents();
            return_value = if ds.is_tuple() {
                quote! { Self::#ident(#(#out_names),*) }
            } else {
                quote! { Self::#ident { #(#out_names),* } }
            };
        },
        EnumVariant::Unit(options) => {
            fields = &[];
            assertions = TokenStream::new();
            let ident = &options.ident;
            return_value = quote! { Self::#ident };
        },
    }

    // TODO: Kind of expensive since the enum is containing all the fields
    // and this is a clone.
    let tla = Input::Enum(en.with_variant(variant));
    // TODO: do not import this way, expose this functionality another way
    let read_body = super::r#struct::generate_body(&tla, &fields);
    quote! {
        #read_body
        #assertions
        Ok(#return_value)
    }
}
