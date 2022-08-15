use super::{
    get_assertions,
    r#struct::{generate_unit_struct, StructGenerator},
    PreludeGenerator,
};
#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::read::{Enum, EnumVariant, Input, UnitEnumField, UnitOnlyEnum};
use crate::parser::EnumErrorMode;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use std::cmp::Ordering;
use std::collections::HashMap;

pub(super) fn generate_unit_enum(
    input: &Input,
    name: Option<&Ident>,
    en: &UnitOnlyEnum,
) -> TokenStream {
    let prelude = PreludeGenerator::new(input)
        .add_imports(name)
        .add_options()
        .add_magic_pre_assertion()
        .finish();

    let read = match &en.repr {
        Some(repr) => generate_unit_enum_repr(repr, &en.fields),
        None => generate_unit_enum_magic(&en.fields),
    };

    quote! {
        #prelude
        #read
    }
}

fn generate_unit_enum_repr(repr: &TokenStream, variants: &[UnitEnumField]) -> TokenStream {
    let clauses = variants.iter().map(|variant| {
        let ident = &variant.ident;
        quote! {
            if #TEMP == Self::#ident as #repr {
                Ok(Self::#ident)
            }
        }
    });

    quote! {
        let #TEMP: #repr = #READ_METHOD(#READER, #OPT, ())?;
        #(#clauses else)* {
            Err(#WITH_CONTEXT(
                #BIN_ERROR::NoVariantMatch {
                    pos: #POS,
                },
                #BACKTRACE_FRAME::OwnedMessage(
                    ::binrw::alloc::format!("Unexpected value for enum: {:?}", #TEMP)
                )
            ))
        }
    }
}

fn generate_unit_enum_magic(variants: &[UnitEnumField]) -> TokenStream {
    // group fields by the type (Kind) of their magic value
    let mut fields_by_magic_type = variants
        .iter()
        .fold(HashMap::new(), |mut fields_by_magic_type, field| {
            let kind = field.magic.as_ref().map(|magic| magic.kind());
            fields_by_magic_type
                .entry(kind)
                .or_insert(vec![])
                .push(field);

            fields_by_magic_type
        })
        .into_iter()
        .collect::<Vec<_>>();

    fields_by_magic_type.sort_by(|a, b| match (&a.0, &b.0) {
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        _ => a.0.cmp(&b.0),
    });

    // for each type (Kind), read and try to match the magic of each field
    let try_each_magic_type = fields_by_magic_type.into_iter().map(|(_, fields)| {
        let amp = fields[0].magic.as_ref().map(|magic| magic.add_ref());

        let matches = fields.iter().map(|field| {
            let ident = &field.ident;

            if let Some(magic) = &field.magic {
                let magic = magic.match_value();
                let condition = if field.pre_assertions.is_empty() {
                    quote! { #magic }
                } else {
                    let pre_assertions =
                        field.pre_assertions.iter().map(|assert| &assert.condition);
                    quote! { #magic if true #(&& (#pre_assertions))* }
                };

                quote! { #condition => Ok(Self::#ident) }
            } else {
                quote! { _ => Ok(Self::#ident) }
            }
        });

        let body = quote! {
            match #amp#READ_METHOD(#READER, #OPT, ())? {
                #(#matches,)*
                _ => Err(#BIN_ERROR::NoVariantMatch { pos: #POS })
            }
        };

        quote! {
            let #TEMP = (|| {
                #body
            })();

            if #TEMP.is_ok() {
                return #TEMP;
            }

            #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#POS))?;
        }
    });

    let return_error = quote! {
        Err(#BIN_ERROR::NoVariantMatch {
               pos: #POS
           })
    };

    quote! {
        #(#try_each_magic_type)*
        #return_error
    }
}

pub(super) fn generate_data_enum(input: &Input, name: Option<&Ident>, en: &Enum) -> TokenStream {
    let return_all_errors = en.error_mode != EnumErrorMode::ReturnUnexpectedError;

    let (create_error_basket, return_error) = if return_all_errors {
        (
            quote! {
                extern crate alloc;
                let mut #ERROR_BASKET: alloc::vec::Vec<(&'static str, #BIN_ERROR)> = alloc::vec::Vec::new();
            },
            quote! {
                Err(#BIN_ERROR::EnumErrors {
                    pos: #POS,
                    variant_errors: #ERROR_BASKET
                })
            },
        )
    } else {
        (
            TokenStream::new(),
            quote! {
                Err(#BIN_ERROR::NoVariantMatch {
                    pos: #POS
                })
            },
        )
    };

    let prelude = PreludeGenerator::new(input)
        .add_imports(name)
        .add_options()
        .add_magic_pre_assertion()
        .reset_position_after_magic()
        .finish();

    let try_each_variant = en.variants.iter().map(|variant| {
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
        #prelude
        #create_error_basket
        #(#try_each_variant)*
        #return_error
    }
}

fn generate_variant_impl(en: &Enum, variant: &EnumVariant) -> TokenStream {
    let input = Input::Struct(variant.clone().into());

    match variant {
        EnumVariant::Variant { ident, options } => StructGenerator::new(&input, options)
            .read_fields(
                None,
                Some(&format!("{}::{}", en.ident.as_ref().unwrap(), &ident)),
            )
            .add_assertions(get_assertions(&en.assertions))
            .return_value(Some(ident))
            .finish(),

        EnumVariant::Unit(options) => generate_unit_struct(&input, None, Some(&options.ident)),
    }
}
