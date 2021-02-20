#![warn(clippy::pedantic)]
#![warn(rust_2018_idioms)]

mod binread_endian;
mod codegen;
mod parser;

use codegen::generate_impl;
use parser::{FromField, StructField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(BinRead, attributes(binread, br))]
pub fn derive_binread_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    generate_impl(&input).into()
}

// TODO: Should not be reparsing every field every time, should iterate the
// fields on the parsed struct tree
fn is_temp(field: &syn::Field) -> bool {
    StructField::from_field(&field)
        .map(|attrs| attrs.temp)
        .unwrap_or(false)
}

fn is_not_binread_attr(attr: &syn::Attribute) -> bool {
    attr.path.get_ident().map_or(true, |ident| ident != "br" && ident != "binread")
}

fn remove_br_attrs(fields: &mut syn::punctuated::Punctuated<syn::Field, syn::Token![,]>) {
    *fields = fields
        .clone()
        .into_pairs()
        .filter_map(|mut field| {
            if is_temp(field.value()) {
                None
            } else {
                field.value_mut().attrs.retain(is_not_binread_attr);
                Some(field)
            }
        })
        .collect()
}

fn remove_field_attrs(fields: &mut syn::Fields) {
    match fields {
        syn::Fields::Named(ref mut fields) => remove_br_attrs(&mut fields.named),
        syn::Fields::Unnamed(ref mut fields) => remove_br_attrs(&mut fields.unnamed),
        syn::Fields::Unit => ()
    }
}

#[proc_macro_attribute]
pub fn derive_binread(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let generated_impl = generate_impl(&input);

    match input.data {
        syn::Data::Struct(ref mut input_struct) => {
            input.attrs.retain(is_not_binread_attr);
            remove_field_attrs(&mut input_struct.fields)
        },
        syn::Data::Enum(ref mut input_enum) => {
            for variant in input_enum.variants.iter_mut() {
                variant.attrs.retain(is_not_binread_attr);
                remove_field_attrs(&mut variant.fields)
            }
        },
        syn::Data::Union(ref mut union) => {
            for field in union.fields.named.iter_mut() {
                field.attrs.retain(is_not_binread_attr);
            }
        },
    }

    input.attrs.retain(is_not_binread_attr);

    quote!(
        #input
        #generated_impl
    ).into()
}
