use crate::{
    codegen::{generate_binwrite_impl, generate_binread_impl},
    parser::{read::is_binread_attr, write, write::is_binwrite_attr, ParseResult},
};

use quote::quote;
use syn::DeriveInput;

fn clean_struct_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !is_binwrite_attr(attr) && !is_binread_attr(attr));
}

// TODO: make this work for `#[binrw::binread]` somehow?
fn has_attr(input: &DeriveInput, attr_name: &str) -> bool {
    input.attrs.iter().any(|attr| {
        attr.path
            .get_ident()
            .map_or(false, |ident| ident == attr_name)
    })
}

pub(crate) fn derive_from_attribute(mut derive_input: DeriveInput) -> proc_macro2::TokenStream {
    let (binwrite_input, generated_impl_wr) = derive_from_input(&derive_input);
    let (binread_input,  generated_impl_rd) = derive_from_input(&derive_input);
    let binwrite_input = binwrite_input.ok();
    let binread_input  = binread_input.ok();
    
    // only clean fields if binread isn't going to be applied after
    ["binread", "binwrite"].into_iter().map(|attr| {
        if !has_attr(&derive_input, attr) {
            clean_struct_attrs(&mut derive_input.attrs);

            match &mut derive_input.data {
                syn::Data::Struct(input_struct) => {
                    clean_field_attrs(&binwrite_input, 0, &mut input_struct.fields);
                }
                syn::Data::Enum(input_enum) => {
                    for (index, variant) in input_enum.variants.iter_mut().enumerate() {
                        clean_struct_attrs(&mut variant.attrs);
                        clean_field_attrs(&binwrite_input, index, &mut variant.fields);
                    }
                }
                syn::Data::Union(union) => {
                    for field in union.fields.named.iter_mut() {
                        clean_struct_attrs(&mut field.attrs);
                    }
                }
            }
        }
    ).collect();

    quote!(
        #derive_input
        #generated_impl
    )
}
fn clean_field_attrs(
    binrw_input: &Option<write::Input>,
    variant_index: usize,
    fields: &mut syn::Fields,
) {
    if let Some(binrw) = binrw {
        let fields = match fields {
            syn::Fields::Named(fields) => &mut fields.named,
            syn::Fields::Unnamed(fields) => &mut fields.unnamed,
            syn::Fields::Unit => return,
        };

        *fields = fields
            .iter_mut()
            .enumerate()
            .filter_map(|(index, value)| {
                if binwrite_input.is_temp_field(variant_index, index) {
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


