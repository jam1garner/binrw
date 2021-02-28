#![warn(clippy::pedantic)]
#![warn(rust_2018_idioms)]

mod codegen;
mod parser;

use codegen::generate_impl;
use parser::Input;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(BinRead, attributes(binread, br))]
pub fn derive_binread_trait(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let binread_input = Input::from_input(&derive_input);
    generate_impl(&derive_input, &binread_input).into()
}

#[proc_macro_attribute]
pub fn derive_binread(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut derive_input = parse_macro_input!(input as DeriveInput);
    let binread_input = Input::from_input(&derive_input);
    let generated_impl = generate_impl(&derive_input, &binread_input);
    let binread_input = binread_input.ok();

    clean_struct_attrs(&mut derive_input.attrs);

    match &mut derive_input.data {
        syn::Data::Struct(input_struct) => {
            clean_field_attrs(&binread_input, 0, &mut input_struct.fields);
        },
        syn::Data::Enum(input_enum) => {
            for (index, variant) in input_enum.variants.iter_mut().enumerate() {
                clean_struct_attrs(&mut variant.attrs);
                clean_field_attrs(&binread_input, index, &mut variant.fields);
            }
        },
        syn::Data::Union(union) => {
            for field in union.fields.named.iter_mut() {
                clean_struct_attrs(&mut field.attrs);
            }
        },
    }

    quote!(
        #derive_input
        #generated_impl
    ).into()
}

fn clean_field_attrs(binread_input: &Option<Input>, variant_index: usize, fields: &mut syn::Fields) {
    if let Some(binread_input) = binread_input {
        let fields = match fields {
            syn::Fields::Named(fields) => &mut fields.named,
            syn::Fields::Unnamed(fields) => &mut fields.unnamed,
            syn::Fields::Unit => return
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

fn clean_struct_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| {
        !attr.path.is_ident("br") && !attr.path.is_ident("binread")
    });
}
