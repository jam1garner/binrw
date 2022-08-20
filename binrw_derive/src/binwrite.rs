use crate::{
    codegen::generate_binwrite_impl,
    parser::{read, read::is_binread_attr, read::is_binwrite_attr, ParseResult},
};

use quote::quote;
use syn::DeriveInput;

#[cfg_attr(coverage_nightly, no_coverage)]
fn clean_struct_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !is_binwrite_attr(attr) && !is_binread_attr(attr));
}

// TODO: make this work for `#[binrw::binread]` somehow?
#[cfg_attr(coverage_nightly, no_coverage)]
fn has_attr(input: &DeriveInput, attr_name: &str) -> bool {
    input.attrs.iter().any(|attr| {
        attr.path
            .get_ident()
            .map_or(false, |ident| ident == attr_name)
    })
}

#[cfg_attr(coverage_nightly, no_coverage)]
pub(crate) fn derive_from_attribute(mut derive_input: DeriveInput) -> proc_macro2::TokenStream {
    let (binwrite_input, generated_impl) = derive_from_input(&derive_input, false);
    let binwrite_input = binwrite_input.ok();

    // TODO: Huh?
    // only clean fields if binread isn't going to be applied after
    if has_attr(&derive_input, "binread") {
        return quote! {
            compile_error!("`binread` and `binwrite` cannot be used together, try using `#[binrw]`");

            #derive_input
            #generated_impl
        };
    }

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

    quote!(
        #derive_input
        #generated_impl
    )
}

pub(crate) fn derive_from_input(
    derive_input: &DeriveInput,
    is_inside_derive: bool,
) -> (ParseResult<read::Input>, proc_macro2::TokenStream) {
    let binwrite_input = read::Input::from_input(derive_input, is_inside_derive, true);
    let generated_impl = generate_binwrite_impl(derive_input, &binwrite_input);
    (binwrite_input, generated_impl)
}

#[cfg_attr(coverage_nightly, no_coverage)]
fn clean_field_attrs(
    binwrite_input: &Option<read::Input>,
    variant_index: usize,
    fields: &mut syn::Fields,
) {
    if let Some(binwrite_input) = binwrite_input {
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
