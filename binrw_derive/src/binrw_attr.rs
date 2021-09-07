use crate::{
    binread,
    binwrite,
    codegen::{generate_binwrite_impl, generate_binread_impl},
    parser::{read, read::is_binread_attr, write, write::is_binwrite_attr, ParseResult},
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
    
    let (binread_input,  generated_impl_rd) = binread::derive_from_input(&derive_input);
    let (binwrite_input, generated_impl_wr) = binwrite::derive_from_input(&derive_input);
    
    let binread_input  = binread_input.ok();
    let binwrite_input = binwrite_input.ok();
    
    quote!(
        #derive_input
        #generated_impl_rd
        #generated_impl_wr
    )
}

pub(crate) fn derive_from_input(
    derive_input: &DeriveInput,
) -> (ParseResult<read::Input>, proc_macro2::TokenStream) {
    
    let binread_input  = read::Input::from_input(derive_input);
    let binwrite_input = write::Input::from_input(derive_input);
    
    let generated_impl_br = generate_binread_impl(derive_input, &binread_input);
    let generated_impl_bw = generate_binwrite_impl(derive_input, &binwrite_input);
    /* this needs to be: `binread_input + binwrite_input` */
    (binread_input, quote!(#generated_impl_br, #generated_impl_br))
}

fn clean_field_attrs(
    binrw_input: &Option<write::Input>,
    variant_index: usize,
    fields: &mut syn::Fields,
) {
    if let Some(binrw) = binrw_input {
        let fields = match fields {
            syn::Fields::Named(fields) => &mut fields.named,
            syn::Fields::Unnamed(fields) => &mut fields.unnamed,
            syn::Fields::Unit => return,
        };

        *fields = fields
            .iter_mut()
            .enumerate()
            .filter_map(|(index, value)| {
                if binrw_input.as_ref().unwrap().is_temp_field(variant_index, index) {
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


