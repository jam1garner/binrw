use crate::{
    binread, binwrite,
    parser::{read, read::is_binread_attr, write, write::is_binwrite_attr},
};

use quote::quote;
use syn::DeriveInput;

#[cfg(not(tarpaulin_include))]
fn clean_struct_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !is_binwrite_attr(attr) && !is_binread_attr(attr));
}

#[cfg(not(tarpaulin_include))]
pub(crate) fn derive_from_attribute(mut derive_input: DeriveInput) -> proc_macro2::TokenStream {
    let (binread_input, generated_impl_rd) = binread::derive_from_input(&derive_input);
    let (binwrite_input, generated_impl_wr) = binwrite::derive_from_input(&derive_input);

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
