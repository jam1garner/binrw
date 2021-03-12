use crate::codegen::sanitization::OPT;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

ident_str! {
    WRITE_COMMENT = from_crate!(binary_template::write_comment);
    WRITE_START_STRUCT = from_crate!(binary_template::write_start_struct);
    WRITE_END_STRUCT = from_crate!(binary_template::write_end_struct);
}

pub(super) fn end() -> TokenStream {
    if cfg!(feature = "debug_template") {
        quote! {
            #WRITE_END_STRUCT(#OPT.variable_name);
        }
    } else {
        <_>::default()
    }
}

pub(super) fn handle_error() -> TokenStream {
    if cfg!(feature = "debug_template") {
        let write_end_struct = end();
        quote! {
            .map_err(|e| {
                #WRITE_COMMENT(&format!("Error: {:?}", e));
                #write_end_struct
                e
            })
        }
    } else {
        <_>::default()
    }
}

pub(super) fn start(struct_name: &Ident) -> TokenStream {
    if cfg!(feature = "debug_template") {
        let struct_name = struct_name.to_string();
        quote! {
            #WRITE_START_STRUCT(#struct_name);
        }
    } else {
        <_>::default()
    }
}
