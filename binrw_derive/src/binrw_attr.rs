use std::collections::HashSet;

use quote::quote;
use syn::DeriveInput;

use crate::codegen::{generate_binread_impl, generate_binwrite_impl};
use crate::parser::{
    read, read::is_binread_attr, write, write::is_binwrite_attr, ParseResult, TempableField,
};

#[cfg(not(tarpaulin_include))]
fn clean_struct_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !is_binwrite_attr(attr) && !is_binread_attr(attr));
}

#[cfg(not(tarpaulin_include))]
pub(crate) fn derive_from_attribute(mut derive_input: DeriveInput) -> proc_macro2::TokenStream {
    let mut binread_input = read::Input::from_input(&derive_input, false);
    let mut binwrite_input = write::Input::from_input(&derive_input, false);

    apply_temp_crossover(&mut binread_input, &mut binwrite_input);

    let generated_impl_rd = generate_binread_impl(&derive_input, &binread_input);
    let generated_impl_wr = generate_binwrite_impl(&derive_input, &binwrite_input);

    let binread_input = binread_input.ok();
    let binwrite_input = binwrite_input.ok();

    clean_struct_attrs(&mut derive_input.attrs);

    match &mut derive_input.data {
        syn::Data::Struct(input_struct) => {
            clean_field_attrs(&binread_input, &binwrite_input, 0, &mut input_struct.fields);
        }
        syn::Data::Enum(input_enum) => {
            for (index, variant) in input_enum.variants.iter_mut().enumerate() {
                clean_struct_attrs(&mut variant.attrs);
                clean_field_attrs(&binread_input, &binwrite_input, index, &mut variant.fields);
            }
        }
        syn::Data::Union(union) => {
            for field in union.fields.named.iter_mut() {
                clean_struct_attrs(&mut field.attrs);
            }
        }
    }

    quote!(
        #derive_input
        #generated_impl_rd
        #generated_impl_wr
    )
}

/// Check the fields of each input and copy temp state to the other input.
fn apply_temp_crossover(
    binread_input: &mut ParseResult<read::Input>,
    binwrite_input: &mut ParseResult<write::Input>,
) {
    let (binread_input, binwrite_input) = match (binread_input, binwrite_input) {
        (ParseResult::Ok(binread), ParseResult::Ok(binwrite)) => (binread, binwrite),
        // We don't need to apply this in the case of Partial because no implementation is
        // generated.
        _ => return,
    };
    match (binread_input, binwrite_input) {
        (read::Input::Struct(binread_struct), write::Input::Struct(binwrite_struct)) => {
            apply_temp_crossover_struct(binread_struct, binwrite_struct);
        }
        (read::Input::Enum(binread_enum), write::Input::Enum(binwrite_enum)) => {
            for (read_variant, write_variant) in binread_enum
                .variants
                .iter_mut()
                .zip(binwrite_enum.variants.iter_mut())
            {
                match (read_variant, write_variant) {
                    (
                        read::EnumVariant::Variant {
                            options: read_struct,
                            ..
                        },
                        write::EnumVariant::Variant {
                            options: write_struct,
                            ..
                        },
                    ) => apply_temp_crossover_struct(read_struct, write_struct),
                    (read::EnumVariant::Unit(_), write::EnumVariant::Unit(_)) => continue,
                    _ => unreachable!("read and write input should always be the same kind"),
                };
            }
        }
        // These don't have temp fields.
        (read::Input::UnitStruct(_), write::Input::UnitStruct(_))
        | (read::Input::UnitOnlyEnum(_), write::Input::UnitOnlyEnum(_)) => {}
        _ => unreachable!("read and write input should always be the same kind"),
    }
}

fn apply_temp_crossover_struct(
    binread_struct: &mut read::Struct,
    binwrite_struct: &mut write::Struct,
) {
    // Index temporary fields
    let read_temporary = extract_temporary_field_names(&binread_struct.fields);
    let write_temporary = extract_temporary_field_names(&binwrite_struct.fields);

    // Iterate the fields again and set temp flags
    set_fields_temporary(&mut binread_struct.fields, &write_temporary);
    set_fields_temporary(&mut binwrite_struct.fields, &read_temporary);
}

fn extract_temporary_field_names<S: TempableField>(fields: &[S]) -> HashSet<syn::Ident> {
    fields
        .iter()
        .filter(|f| f.is_temp_for_crossover())
        .map(|f| f.ident().clone())
        .collect()
}

fn set_fields_temporary<S: TempableField>(fields: &mut [S], temporary_names: &HashSet<syn::Ident>) {
    for field in fields {
        if temporary_names.contains(field.ident()) {
            field.set_crossover_temp(true);
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn clean_field_attrs(
    binread_input: &Option<read::Input>,
    binwrite_input: &Option<write::Input>,
    variant_index: usize,
    fields: &mut syn::Fields,
) {
    if let (Some(binread_input), Some(binwrite_input)) = (binread_input, binwrite_input) {
        let fields = match fields {
            syn::Fields::Named(fields) => &mut fields.named,
            syn::Fields::Unnamed(fields) => &mut fields.unnamed,
            syn::Fields::Unit => return,
        };

        *fields = fields
            .iter_mut()
            .enumerate()
            .filter_map(|(index, value)| {
                if binread_input.is_temp_field(variant_index, index)
                    || binwrite_input.is_temp_field(variant_index, index)
                {
                    None
                } else {
                    let mut value = value.clone();
                    clean_struct_attrs(&mut value.attrs);
                    Some(value)
                }
            })
            .collect();
    }
}
