use crate::{
    codegen::{generate_binread_impl, generate_binwrite_impl},
    combine_error,
    parser::{Enum, EnumVariant, FieldMode, Input, ParseResult, Struct, StructField},
    Options,
};
use quote::quote;
use std::collections::HashSet;
use syn::{spanned::Spanned, DeriveInput};

pub(crate) fn derive_from_attribute(mut derive_input: DeriveInput) -> proc_macro2::TokenStream {
    let mut binread_input = Input::from_input(
        &derive_input,
        Options {
            derive: false,
            write: false,
        },
    );
    let mut binwrite_input = Input::from_input(
        &derive_input,
        Options {
            derive: false,
            write: true,
        },
    );

    // TODO: Make this not bad
    if let Some(error) = apply_temp_crossover(&mut binread_input, &mut binwrite_input) {
        binwrite_input = ParseResult::Partial(binwrite_input.unwrap_tuple().0, error);
    }

    let generated_read_impl = generate_binread_impl(&derive_input, &binread_input);
    let generated_write_impl = generate_binwrite_impl(&derive_input, &binwrite_input);

    // Since temporary fields must be synchronised between binread and binwrite,
    // the same cleaning mechanism can be used as-if there was only one input
    super::clean_attr(&mut derive_input, &binread_input.ok());

    quote!(
        #derive_input
        #generated_read_impl
        #generated_write_impl
    )
}

/// Check the fields of each input and copy temp state to the other input.
fn apply_temp_crossover(
    binread_result: &mut ParseResult<Input>,
    binwrite_result: &mut ParseResult<Input>,
) -> Option<syn::Error> {
    let (binread_input, binwrite_input) = match (binread_result, binwrite_result) {
        (ParseResult::Ok(binread), ParseResult::Ok(binwrite)) => (binread, binwrite),
        // We don't need to apply this in the case of Partial because no
        // implementation is generated.
        _ => return None,
    };

    match (binread_input, binwrite_input) {
        (Input::Struct(binread_struct), Input::Struct(binwrite_struct)) => {
            apply_temp_crossover_struct(binread_struct, binwrite_struct)
        }
        (Input::Enum(binread_enum), Input::Enum(binwrite_enum)) => {
            apply_temp_crossover_enum(binread_enum, binwrite_enum)
        }
        // These don't have temp fields.
        (Input::UnitStruct(_), Input::UnitStruct(_))
        | (Input::UnitOnlyEnum(_), Input::UnitOnlyEnum(_)) => None,
        _ => unreachable!("read and write input should always be the same kind"),
    }
}

fn apply_temp_crossover_enum(
    binread_enum: &mut Enum,
    binwrite_enum: &mut Enum,
) -> Option<syn::Error> {
    let mut all_errors = None::<syn::Error>;
    for (read_variant, write_variant) in binread_enum
        .variants
        .iter_mut()
        .zip(binwrite_enum.variants.iter_mut())
    {
        match (read_variant, write_variant) {
            (
                EnumVariant::Variant {
                    options: read_struct,
                    ..
                },
                EnumVariant::Variant {
                    options: write_struct,
                    ..
                },
            ) => {
                if let Some(error) = apply_temp_crossover_struct(read_struct, write_struct) {
                    combine_error(&mut all_errors, error);
                }
            }
            (EnumVariant::Unit(_), EnumVariant::Unit(_)) => continue,
            _ => unreachable!("read and write input should always be the same kind"),
        };
    }
    all_errors
}

fn apply_temp_crossover_struct(
    binread_struct: &mut Struct,
    binwrite_struct: &mut Struct,
) -> Option<syn::Error> {
    // Index temporary fields
    let read_temporary = extract_temporary_field_names(&binread_struct.fields, false);
    let write_temporary = extract_temporary_field_names(&binwrite_struct.fields, true);

    if let Some(error) = validate_fields_temporary(&binwrite_struct.fields, &read_temporary) {
        return Some(error);
    }

    // Iterate the fields again and set temp flags
    set_fields_temporary(&mut binread_struct.fields, &write_temporary);
    set_fields_temporary(&mut binwrite_struct.fields, &read_temporary);
    None
}

fn validate_fields_temporary(
    fields: &[StructField],
    read_temporary: &HashSet<syn::Ident>,
) -> Option<syn::Error> {
    let mut all_errors = None::<syn::Error>;
    for field in fields {
        if read_temporary.contains(&field.ident)
            && !matches!(field.read_mode, FieldMode::Calc(_) | FieldMode::Default)
        {
            combine_error(
                &mut all_errors,
                syn::Error::new(
                    field.field.span(),
                    "`#[br(temp)]` is invalid without a corresponding `#[bw(ignore)]` or `#[bw(calc)]`",
                ),
            );
        }
    }
    all_errors
}

fn extract_temporary_field_names(fields: &[StructField], for_write: bool) -> HashSet<syn::Ident> {
    fields
        .iter()
        .filter(|f| f.is_temp(for_write))
        .map(|f| f.ident.clone())
        .collect()
}

fn set_fields_temporary(fields: &mut [StructField], temporary_names: &HashSet<syn::Ident>) {
    for field in fields {
        if temporary_names.contains(&field.ident) {
            field.force_temp();
        }
    }
}
