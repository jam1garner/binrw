use super::{
    get_assertions,
    r#struct::{generate_unit_struct, StructGenerator},
    PreludeGenerator,
};
#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::{Enum, EnumErrorMode, EnumVariant, Input, UnitEnumField, UnitOnlyEnum};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn generate_unit_enum(input: &Input, en: &UnitOnlyEnum) -> TokenStream {
    match &en.repr {
        Some(repr) => generate_unit_enum_repr(input, repr, &en.fields),
        None => generate_unit_enum_magic(input, en, &en.fields),
    }
}

fn generate_unit_enum_repr(
    input: &Input,
    repr: &TokenStream,
    variants: &[UnitEnumField],
) -> TokenStream {
    let clauses = variants.iter().map(|variant| {
        let ident = &variant.ident;
        quote! {
            if #TEMP == Self::#ident as #repr {
                Ok(Self::#ident)
            }
        }
    });

    let prelude = PreludeGenerator::new(input).add_options().finish();

    quote! {
        #prelude
        let #TEMP: #repr = #READ_METHOD(#READER, #OPT, ())?;
        #(#clauses else)* {
            Err(#BIN_ERROR::NoVariantMatch {
                pos: #POS,
            })
        }
    }
}

fn generate_unit_enum_magic(
    input: &Input,
    en: &UnitOnlyEnum,
    variants: &[UnitEnumField],
) -> TokenStream {
    let prelude = PreludeGenerator::new(input)
        .add_imports()
        .add_options()
        .finish();

    let matches = variants.iter().filter_map(|field| {
        if let Some(magic) = &field.magic {
            let ident = &field.ident;
            let magic = magic.match_value();
            let condition = if field.pre_assertions.is_empty() {
                quote! { #magic }
            } else {
                let pre_assertions = field.pre_assertions.iter().map(|assert| &assert.condition);
                quote! { #magic if true #(&& (#pre_assertions))* }
            };
            Some(quote! { #condition => Ok(Self::#ident) })
        } else {
            None
        }
    });

    let amp = en
        .expected_field_magic
        .as_ref()
        .map(|magic| magic.add_ref());

    quote! {
        #prelude
        match #amp#READ_METHOD(#READER, #OPT, ())? {
            #(#matches,)*
            _ => Err(#BIN_ERROR::NoVariantMatch { pos: #POS })
        }
    }
}

pub(super) fn generate_data_enum(en: &Enum) -> TokenStream {
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
        #create_error_basket
        #(#try_each_variant)*
        #return_error
    }
}

fn generate_variant_impl(en: &Enum, variant: &EnumVariant) -> TokenStream {
    // TODO: Kind of expensive since the enum is containing all the fields
    // and this is a clone.
    let input = Input::Enum(en.with_variant(variant));

    match variant {
        EnumVariant::Variant { ident, options } => StructGenerator::new(&input, &options)
            .read_fields()
            .add_assertions(get_assertions(&en.assertions))
            .return_value(Some(ident))
            .finish(),

        EnumVariant::Unit(options) => generate_unit_struct(&input, Some(&options.ident)),
    }
}
