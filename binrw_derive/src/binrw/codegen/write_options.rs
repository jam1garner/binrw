mod r#enum;
mod prelude;
mod r#struct;
mod struct_field;

use super::get_map_err;
use crate::binrw::{
    codegen::sanitization::{OPT, POS, SEEK_TRAIT, WRITER, WRITE_METHOD},
    parser::{Input, Map},
};
use proc_macro2::TokenStream;
use quote::quote;
use r#enum::{generate_data_enum, generate_unit_enum};
use r#struct::generate_struct;
use syn::{spanned::Spanned, Ident};

pub(crate) fn generate(input: &Input, derive_input: &syn::DeriveInput) -> TokenStream {
    let name = Some(&derive_input.ident);
    let inner = match input.map() {
        Map::None => match input {
            Input::UnitStruct(s) | Input::Struct(s) => generate_struct(input, name, s),
            Input::Enum(e) => generate_data_enum(input, name, e),
            Input::UnitOnlyEnum(e) => generate_unit_enum(input, name, e),
        },
        Map::Try(map) | Map::Map(map) => generate_map(input, name, map),
    };

    quote! {
        let #POS = #SEEK_TRAIT::stream_position(#WRITER)?;
        #inner

        Ok(())
    }
}

fn generate_map(input: &Input, name: Option<&Ident>, map: &TokenStream) -> TokenStream {
    let map_try = input.map().is_try().then(|| {
        let map_err = get_map_err(POS, map.span());
        quote! { #map_err? }
    });
    let write_data = quote! {
        #WRITE_METHOD(
            &((#map)(self) #map_try),
            #WRITER,
            #OPT,
            ()
        )?;
    };

    let magic = input.magic();
    let endian = input.endian();
    prelude::PreludeGenerator::new(write_data, Some(input), name)
        .prefix_magic(magic)
        .prefix_endian(endian)
        .prefix_imports()
        .finish()
}
