use super::{
    r#struct::{generate_unit_struct, StructGenerator},
    PreludeGenerator,
};
use crate::binrw::{
    codegen::sanitization::{
        ALL_EOF, BACKTRACE_FRAME, BIN_ERROR, ERROR_BASKET, NOT_ENOUGH_BYTES, OPT, POS, READER,
        READ_METHOD, RESTORE_POSITION_VARIANT, TEMP, WITH_CONTEXT,
    },
    parser::{Enum, EnumErrorMode, EnumVariant, Input, UnitEnumField, UnitOnlyEnum},
};
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
        .add_endian()
        .add_magic_pre_assertion()
        .finish();

    let read = match en.map.as_repr() {
        Some(repr) => generate_unit_enum_repr(&input.stream_ident_or(READER), repr, &en.fields),
        None => generate_unit_enum_magic(&input.stream_ident_or(READER), &en.fields),
    };

    quote! {
        #prelude
        #read
    }
}

fn generate_unit_enum_repr(
    reader_var: &TokenStream,
    repr: &TokenStream,
    variants: &[UnitEnumField],
) -> TokenStream {
    let clauses = variants.iter().map(|variant| {
        let ident = &variant.ident;
        let pre_assertions = variant
            .pre_assertions
            .iter()
            .map(|assert| &assert.condition);

        quote! {
            if #TEMP == Self::#ident as #repr #(&& (#pre_assertions))* {
                ::core::result::Result::Ok(Self::#ident)
            }
        }
    });

    quote! {
        let #TEMP: #repr = #READ_METHOD(#reader_var, #OPT, ())?;
        #(#clauses else)* {
            ::core::result::Result::Err(#WITH_CONTEXT(
                #BIN_ERROR::NoVariantMatch {
                    pos: #POS,
                },
                #BACKTRACE_FRAME::Message({
                    extern crate alloc;
                    ::core::convert::Into::into(alloc::format!("Unexpected value for enum: {:?}", #TEMP))
                })
            ))
        }
    }
}

fn generate_unit_enum_magic(reader_var: &TokenStream, variants: &[UnitEnumField]) -> TokenStream {
    // group fields by the type (Kind) of their magic value, preserve the order
    let group_by_magic_type = variants.iter().fold(
        Vec::new(),
        |mut group_by_magic_type: Vec<(_, Vec<_>)>, field| {
            let kind = field.magic.as_ref().map(|magic| magic.kind());
            let last = group_by_magic_type.last_mut();
            match last {
                // if the current field's magic kind is the same as the previous one
                // then add the current field to the same group
                // if the magic kind is none then it's a wildcard, just add it to the previous group
                Some((last_kind, last_vec)) if kind.is_none() || *last_kind == kind => {
                    last_vec.push(field);
                }
                // otherwise if the vector is empty
                // or the last field's magic kind is different
                // then create a new group
                _ => group_by_magic_type.push((kind, vec![field])),
            }

            group_by_magic_type
        },
    );

    // for each type (Kind), read and try to match the magic of each field
    let try_each_magic_type = group_by_magic_type.into_iter().map(|(_kind, fields)| {
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

                quote! { #condition => ::core::result::Result::Ok(Self::#ident) }
            } else {
                quote! { _ => ::core::result::Result::Ok(Self::#ident) }
            }
        });

        let body = quote! {
            match #amp #READ_METHOD(#reader_var, #OPT, ())? {
                #(#matches,)*
                _ => ::core::result::Result::Err(#BIN_ERROR::NoVariantMatch { pos: #POS })
            }
        };

        quote! {
            match (|| {
                #body
            })() {
                v @ ::core::result::Result::Ok(_) => return v,
                ::core::result::Result::Err(#TEMP) => {
                    #ALL_EOF &= #TEMP.is_eof();
                    #RESTORE_POSITION_VARIANT(#reader_var, #POS, #TEMP)?;
                }
            }
        }
    });

    let return_error = quote! {
        if #ALL_EOF {
            ::core::result::Result::Err(#NOT_ENOUGH_BYTES())
        } else {
            ::core::result::Result::Err(#BIN_ERROR::NoVariantMatch {
                pos: #POS
            })
        }
    };

    quote! {
        let mut #ALL_EOF = true;
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
                ::core::result::Result::Err(#BIN_ERROR::EnumErrors {
                    pos: #POS,
                    variant_errors: #ERROR_BASKET
                })
            },
        )
    } else {
        (
            TokenStream::new(),
            quote! {
                ::core::result::Result::Err(#BIN_ERROR::NoVariantMatch {
                    pos: #POS
                })
            },
        )
    };

    let prelude = PreludeGenerator::new(input)
        .add_imports(name)
        .add_endian()
        .add_magic_pre_assertion()
        .reset_position_after_magic()
        .finish();

    let reader_var = input.stream_ident_or(READER);

    let try_each_variant = en.variants.iter().map(|variant| {
        let body = generate_variant_impl(en, variant);

        let handle_error = if return_all_errors {
            let name = variant.ident().to_string();
            quote! {
                #ERROR_BASKET.push((#name, #TEMP));
            }
        } else {
            TokenStream::new()
        };

        quote! {
            match (|| {
                #body
            })() {
                ok @ ::core::result::Result::Ok(_) => return ok,
                ::core::result::Result::Err(error) => {
                    #RESTORE_POSITION_VARIANT(#reader_var, #POS, error).map(|#TEMP| {
                        #handle_error
                    })?;
                }
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
            .initialize_value_with_assertions(Some(ident), &en.assertions)
            .return_value()
            .finish(),

        EnumVariant::Unit(options) => generate_unit_struct(&input, None, Some(&options.ident)),
    }
}
