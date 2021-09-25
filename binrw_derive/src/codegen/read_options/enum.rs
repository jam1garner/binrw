use super::{
    get_assertions,
    r#struct::{generate_unit_struct, StructGenerator},
    PreludeGenerator,
};
#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::read::{Enum, EnumVariant, Input, UnitEnumField, UnitOnlyEnum};
use crate::parser::{EnumErrorMode, Imports};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

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
        None => generate_unit_enum_magic(en, &en.fields),
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
            Err(#BIN_ERROR::NoVariantMatch {
                pos: #POS,
            })
        }
    }
}

fn generate_unit_enum_magic(en: &UnitOnlyEnum, variants: &[UnitEnumField]) -> TokenStream {
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
        match #amp#READ_METHOD(#READER, #OPT, ())? {
            #(#matches,)*
            _ => Err(#BIN_ERROR::NoVariantMatch { pos: #POS })
        }
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
    // TODO: Kind of expensive since the enum is containing all the fields
    // and this is a clone.
    let mut new_enum = en.with_variant(variant);
    // Drop imports, we already have them in scope
    new_enum.imports = Imports::None;
    let input = Input::Enum(new_enum);

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
