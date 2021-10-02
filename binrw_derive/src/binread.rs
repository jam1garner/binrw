use crate::{
    codegen::generate_binread_impl,
    parser::{read, read::is_binread_attr, write::is_binwrite_attr, ParseResult},
};

use quote::quote;
use syn::DeriveInput;

#[cfg(not(tarpaulin_include))]
fn clean_struct_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !is_binwrite_attr(attr) && !is_binread_attr(attr));
}

// TODO: make this work for `#[binrw::binwrite]` somehow?
#[cfg(not(tarpaulin_include))]
fn has_attr(input: &DeriveInput, attr_name: &str) -> bool {
    input.attrs.iter().any(|attr| {
        attr.path
            .get_ident()
            .map_or(false, |ident| ident == attr_name)
    })
}

#[cfg(not(tarpaulin_include))]
pub(crate) fn derive_from_attribute(mut derive_input: DeriveInput) -> proc_macro2::TokenStream {
    let (binread_input, generated_impl) = derive_from_input(&derive_input, false);
    let binread_input = binread_input.ok();

    // only clean fields if binwrite isn't going to be applied after
    if has_attr(&derive_input, "binwrite") {
        return quote! {
            compile_error!("`binread` and `binwrite` cannot be used togeter, try using `#[binrw]`");

            #derive_input
            #generated_impl
        };
    }

    clean_struct_attrs(&mut derive_input.attrs);

    match &mut derive_input.data {
        syn::Data::Struct(input_struct) => {
            clean_field_attrs(&binread_input, 0, &mut input_struct.fields);
        }
        syn::Data::Enum(input_enum) => {
            for (index, variant) in input_enum.variants.iter_mut().enumerate() {
                clean_struct_attrs(&mut variant.attrs);
                clean_field_attrs(&binread_input, index, &mut variant.fields);
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
        #generated_impl
    )
}

pub(crate) fn derive_from_input(
    derive_input: &DeriveInput,
    is_inside_derive: bool,
) -> (ParseResult<read::Input>, proc_macro2::TokenStream) {
    let binread_input = read::Input::from_input(derive_input, is_inside_derive);
    let generated_impl = generate_binread_impl(derive_input, &binread_input);
    (binread_input, generated_impl)
}

#[cfg(not(tarpaulin_include))]
fn clean_field_attrs(
    binread_input: &Option<read::Input>,
    variant_index: usize,
    fields: &mut syn::Fields,
) {
    if let Some(binread_input) = binread_input {
        let fields = match fields {
            syn::Fields::Named(fields) => &mut fields.named,
            syn::Fields::Unnamed(fields) => &mut fields.unnamed,
            syn::Fields::Unit => return,
        };

        *fields = fields
            .iter_mut()
            .enumerate()
            .filter_map(|(index, value)| {
                if binread_input.is_temp_field(variant_index, index) {
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
