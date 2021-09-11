use crate::parser::{write::Input, Map};
use proc_macro2::TokenStream;
use quote::quote;

mod r#struct;
use r#struct::generate_struct;

mod prelude;
mod struct_field;

mod r#enum;
use r#enum::{generate_data_enum, generate_unit_enum};

pub(crate) fn generate(input: &Input, derive_input: &syn::DeriveInput) -> TokenStream {
    let name = Some(&derive_input.ident);
    let inner = match input.map() {
        Map::None => match input {
            Input::UnitStruct(_) => todo!(), //generate_unit_struct(input, name, None),
            Input::Struct(s) => generate_struct(input, name, s),
            Input::Enum(e) => generate_data_enum(input, name, e),
            Input::UnitOnlyEnum(e) => generate_unit_enum(input, name, e),
        },
        Map::Try(_map) => todo!(),
        Map::Map(_map) => todo!(),
    };

    //quote! {
    //    let #POS = #SEEK_TRAIT::stream_position(#READER)?;
    //    (|| {
    //        #inner
    //    })().or_else(|error| {
    //        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#POS))?;
    //        Err(error)
    //    })
    //}

    quote! {
        #inner

        Ok(())
    }
}
