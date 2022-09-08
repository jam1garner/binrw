#![warn(clippy::pedantic)]
#![warn(rust_2018_idioms)]
#![cfg_attr(nightly, feature(proc_macro_span))]
#![cfg_attr(coverage_nightly, feature(no_coverage))]

extern crate alloc;

mod binrw;
mod fn_helper;
mod meta_types;
mod named_args;
mod result;
pub(crate) mod util;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn binread(attr: TokenStream, input: TokenStream) -> TokenStream {
    binrw::derive_from_attribute(&attr, input, false)
}

#[proc_macro_derive(BinRead, attributes(br, brw))]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn binread_derive(input: TokenStream) -> TokenStream {
    binrw::derive_from_input(
        parse_macro_input!(input as DeriveInput),
        binrw::Options {
            derive: true,
            write: false,
        },
    )
    .into()
}

#[proc_macro_attribute]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn binrw(attr: TokenStream, input: TokenStream) -> TokenStream {
    if attr.to_string() == "ignore" {
        input
    } else {
        binrw::binrw_derive(parse_macro_input!(input as DeriveInput)).into()
    }
}

#[proc_macro_derive(BinWrite, attributes(bw, brw))]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn binwrite_derive(input: TokenStream) -> TokenStream {
    binrw::derive_from_input(
        parse_macro_input!(input as DeriveInput),
        binrw::Options {
            derive: true,
            write: true,
        },
    )
    .into()
}

#[proc_macro_attribute]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn binwrite(attr: TokenStream, input: TokenStream) -> TokenStream {
    binrw::derive_from_attribute(&attr, input, true)
}

#[proc_macro_derive(NamedArgs, attributes(named_args))]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn named_args_derive(input: TokenStream) -> TokenStream {
    named_args::derive_from_input(parse_macro_input!(input as DeriveInput)).into()
}

#[proc_macro_attribute]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn parser(attr: TokenStream, input: TokenStream) -> TokenStream {
    fn_helper::derive_from_attribute::<false>(attr, input)
}

#[proc_macro_attribute]
#[cfg_attr(coverage_nightly, no_coverage)]
pub fn writer(attr: TokenStream, input: TokenStream) -> TokenStream {
    fn_helper::derive_from_attribute::<true>(attr, input)
}

fn combine_error(all_errors: &mut Option<syn::Error>, new_error: syn::Error) {
    if let Some(all_errors) = all_errors {
        all_errors.combine(new_error);
    } else {
        *all_errors = Some(new_error);
    }
}
