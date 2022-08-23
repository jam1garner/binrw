#![warn(clippy::pedantic)]
#![warn(rust_2018_idioms)]
#![allow(
    clippy::expl_impl_clone_on_copy,
    clippy::large_enum_variant,
    clippy::redundant_closure_for_method_calls
)]
#![cfg_attr(all(nightly, not(coverage)), feature(proc_macro_span))]
#![cfg_attr(all(nightly, coverage), feature(no_coverage))]

#[cfg(all(nightly, not(coverage)))]
mod backtrace;
mod binrw_attr;
mod codegen;
mod named_args;
mod parser;

use codegen::{generate_binread_impl, generate_binwrite_impl};
use parser::{Input, ParseResult};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(BinRead, attributes(binread, br, brw))]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn derive_binread_trait(input: TokenStream) -> TokenStream {
    derive_from_input(
        parse_macro_input!(input as DeriveInput),
        Options {
            derive: true,
            write: false,
        },
    )
    .into()
}

#[proc_macro_attribute]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn binread(_: TokenStream, input: TokenStream) -> TokenStream {
    derive_from_input(
        parse_macro_input!(input as DeriveInput),
        Options {
            derive: false,
            write: false,
        },
    )
    .into()
}

#[proc_macro_derive(BinWrite, attributes(binwrite, bw, brw))]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn derive_binwrite_trait(input: TokenStream) -> TokenStream {
    derive_from_input(
        parse_macro_input!(input as DeriveInput),
        Options {
            derive: true,
            write: true,
        },
    )
    .into()
}

#[proc_macro_attribute]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn binwrite(_: TokenStream, input: TokenStream) -> TokenStream {
    derive_from_input(
        parse_macro_input!(input as DeriveInput),
        Options {
            derive: false,
            write: true,
        },
    )
    .into()
}

#[proc_macro_attribute]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn binrw(_: TokenStream, input: TokenStream) -> TokenStream {
    binrw_attr::derive_from_attribute(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_derive(BinrwNamedArgs, attributes(named_args))]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn derive_binrw_named_args(input: TokenStream) -> TokenStream {
    named_args::derive_from_attribute(parse_macro_input!(input as DeriveInput)).into()
}

/// Input handling options.
#[derive(Clone, Copy)]
struct Options {
    /// If `true`, the input is from a `#[derive]` instead of an attribute.
    derive: bool,
    /// If `true`, the input is for `BinWrite`.
    write: bool,
}

fn combine_error(all_errors: &mut Option<syn::Error>, new_error: syn::Error) {
    if let Some(all_errors) = all_errors {
        all_errors.combine(new_error);
    } else {
        *all_errors = Some(new_error);
    }
}

#[cfg_attr(coverage_nightly, no_coverage)]
fn clean_attr(derive_input: &mut DeriveInput, binrw_input: &Option<Input>) {
    clean_struct_attrs(&mut derive_input.attrs);

    match &mut derive_input.data {
        syn::Data::Struct(st) => {
            clean_field_attrs(binrw_input, 0, &mut st.fields);
        }
        syn::Data::Enum(en) => {
            for (index, variant) in en.variants.iter_mut().enumerate() {
                clean_struct_attrs(&mut variant.attrs);
                clean_field_attrs(binrw_input, index, &mut variant.fields);
            }
        }
        syn::Data::Union(union) => {
            for field in union.fields.named.iter_mut() {
                clean_struct_attrs(&mut field.attrs);
            }
        }
    }
}

#[cfg_attr(coverage_nightly, no_coverage)]
fn clean_field_attrs(input: &Option<Input>, variant_index: usize, fields: &mut syn::Fields) {
    if let Some(input) = input {
        let fields = match fields {
            syn::Fields::Named(fields) => &mut fields.named,
            syn::Fields::Unnamed(fields) => &mut fields.unnamed,
            syn::Fields::Unit => return,
        };

        *fields = fields
            .iter_mut()
            .enumerate()
            .filter_map(|(index, value)| {
                if input.is_temp_field(variant_index, index) {
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

#[cfg_attr(coverage_nightly, no_coverage)]
fn clean_struct_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !is_binwrite_attr(attr) && !is_binread_attr(attr));
}

#[cfg_attr(coverage_nightly, no_coverage)]
fn derive_from_input(mut derive_input: DeriveInput, options: Options) -> proc_macro2::TokenStream {
    let (binrw_input, generated_impl) = parse(&derive_input, options);
    let binrw_input = binrw_input.ok();

    // only clean fields if binwrite isn't going to be applied after
    if has_attr(
        &derive_input,
        if options.write { "binread" } else { "binwrite" },
    ) {
        return quote! {
            compile_error!("cannot combine `#[binread]` and `#[binwrite]`; use `#[binrw]` instead");

            #derive_input
            #generated_impl
        };
    }

    if options.derive {
        generated_impl
    } else {
        clean_struct_attrs(&mut derive_input.attrs);

        match &mut derive_input.data {
            syn::Data::Struct(st) => {
                clean_field_attrs(&binrw_input, 0, &mut st.fields);
            }
            syn::Data::Enum(en) => {
                for (index, variant) in en.variants.iter_mut().enumerate() {
                    clean_struct_attrs(&mut variant.attrs);
                    clean_field_attrs(&binrw_input, index, &mut variant.fields);
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
}

// TODO: make this work for `#[binrw::binwrite]` somehow?
#[cfg_attr(coverage_nightly, no_coverage)]
fn has_attr(input: &DeriveInput, attr_name: &str) -> bool {
    input.attrs.iter().any(|attr| {
        attr.path
            .get_ident()
            .map_or(false, |ident| ident == attr_name)
    })
}

fn is_binread_attr(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("br") || attr.path.is_ident("brw")
}

fn is_binwrite_attr(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("bw") || attr.path.is_ident("brw")
}

fn parse(
    derive_input: &DeriveInput,
    options: Options,
) -> (ParseResult<Input>, proc_macro2::TokenStream) {
    let binrw_input = Input::from_input(derive_input, options);
    let generated_impl = if options.write {
        generate_binwrite_impl(derive_input, &binrw_input)
    } else {
        generate_binread_impl(derive_input, &binrw_input)
    };
    (binrw_input, generated_impl)
}

#[cfg(test)]
#[cfg(coverage)]
#[cfg_attr(coverage_nightly, no_coverage)]
#[test]
fn derive_code_coverage_for_tool() {
    use runtime_macros_derive::emulate_derive_expansion_fallible;
    use std::{env, fs};

    let derive_tests_folder = env::current_dir()
        .unwrap()
        .join("..")
        .join("binrw")
        .join("tests")
        .join("derive");

    let mut run_success = true;
    for entry in fs::read_dir(derive_tests_folder).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let file = fs::File::open(entry.path()).unwrap();
            if emulate_derive_expansion_fallible(file, "BinRead", |input| {
                parse(
                    &input,
                    Options {
                        derive: true,
                        write: false,
                    },
                )
                .1
            })
            .is_err()
            {
                run_success = false;
            }
        }
    }

    assert!(run_success)
}

#[cfg(test)]
#[cfg(coverage)]
#[cfg_attr(coverage_nightly, no_coverage)]
#[test]
fn derive_binwrite_code_coverage_for_tool() {
    use runtime_macros_derive::emulate_derive_expansion_fallible;
    use std::{env, fs};

    let derive_tests_folder = env::current_dir()
        .unwrap()
        .join("..")
        .join("binrw/tests/derive/write");

    let mut run_success = true;
    for entry in fs::read_dir(derive_tests_folder).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let file = fs::File::open(entry.path()).unwrap();
            if emulate_derive_expansion_fallible(file, "BinWrite", |input| {
                parse(
                    &input,
                    Options {
                        derive: true,
                        write: true,
                    },
                )
                .1
            })
            .is_err()
            {
                run_success = false;
            }
        }
    }

    assert!(run_success)
}
