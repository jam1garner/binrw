#[cfg(feature = "verbose-backtrace")]
mod backtrace;
mod codegen;
mod combiner;
mod parser;

use codegen::generate_impl;
pub(crate) use combiner::derive as binrw_derive;
use parser::{Input, ParseResult};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Input handling options.
#[derive(Clone, Copy)]
pub(super) struct Options {
    /// If `true`, the input is from a `#[derive]` instead of an attribute.
    pub(super) derive: bool,
    /// If `true`, the input is for `BinWrite`.
    pub(super) write: bool,
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn clean_attr(derive_input: &mut DeriveInput, binrw_input: Option<&Input>) {
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
            for field in &mut union.fields.named {
                clean_struct_attrs(&mut field.attrs);
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn clean_field_attrs(input: Option<&Input>, variant_index: usize, fields: &mut syn::Fields) {
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

#[cfg_attr(coverage_nightly, coverage(off))]
fn clean_struct_attrs(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|attr| !is_binwrite_attr(attr) && !is_binread_attr(attr));
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn derive_from_attribute(
    attr: &TokenStream,
    input: TokenStream,
    write: bool,
) -> TokenStream {
    if attr.to_string() == "ignore" {
        return input;
    }

    let mut derive_input = parse_macro_input!(input as DeriveInput);

    let mut mixed_rw = false;
    let opposite_attr = if write { "binread" } else { "binwrite" };
    for attr in &mut derive_input.attrs {
        if let Some(seg) = attr.path.segments.last() {
            let ident = &seg.ident;
            if ident == "binrw" || ident == "binread" || ident == "binwrite" {
                attr.tokens = quote! { (ignore) };

                if ident == "binrw" || ident == opposite_attr {
                    mixed_rw = true;
                }
            }
        }
    }

    if mixed_rw {
        combiner::derive(derive_input)
    } else {
        derive_from_input(
            derive_input,
            Options {
                derive: false,
                write,
            },
        )
    }
    .into()
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn derive_from_input(
    mut derive_input: DeriveInput,
    options: Options,
) -> proc_macro2::TokenStream {
    let (binrw_input, generated_impl) = parse(&derive_input, options);
    let binrw_input = binrw_input.ok();

    if options.derive {
        generated_impl
    } else {
        clean_struct_attrs(&mut derive_input.attrs);

        match &mut derive_input.data {
            syn::Data::Struct(st) => {
                clean_field_attrs(binrw_input.as_ref(), 0, &mut st.fields);
            }
            syn::Data::Enum(en) => {
                for (index, variant) in en.variants.iter_mut().enumerate() {
                    clean_struct_attrs(&mut variant.attrs);
                    clean_field_attrs(binrw_input.as_ref(), index, &mut variant.fields);
                }
            }
            syn::Data::Union(union) => {
                for field in &mut union.fields.named {
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
        generate_impl::<true>(derive_input, &binrw_input)
    } else {
        generate_impl::<false>(derive_input, &binrw_input)
    };
    (binrw_input, generated_impl)
}

#[cfg(coverage)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[test]
fn derive_binread_code_coverage_for_tool() {
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

#[cfg(coverage)]
#[cfg_attr(coverage_nightly, coverage(off))]
#[test]
fn derive_binwrite_code_coverage_for_tool() {
    use runtime_macros_derive::emulate_derive_expansion_fallible;
    use std::{env, fs};

    let derive_tests_folder = env::current_dir()
        .unwrap()
        .join("..")
        .join("binrw")
        .join("tests")
        .join("derive")
        .join("write");

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
