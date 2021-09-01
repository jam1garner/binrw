use crate::parser::write::Input;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate(_input: &Input, _derive_input: &syn::DeriveInput) -> TokenStream {
    //let name = Some(&derive_input.ident);
    //let inner = match input.map() {
    //    Map::None => match input {
    //        Input::UnitStruct(_) => generate_unit_struct(input, name, None),
    //        Input::Struct(s) => generate_struct(input, name, s),
    //        Input::Enum(e) => generate_data_enum(input, name, e),
    //        Input::UnitOnlyEnum(e) => generate_unit_enum(input, name, e),
    //    },
    //    Map::Try(map) => {
    //        let map_err = get_map_err(POS);
    //        quote! {
    //            #READ_METHOD(#READER, #OPT, #ARGS).and_then(|value| {
    //                #map(value)#map_err
    //            })
    //        }
    //    }
    //    Map::Map(map) => quote! {
    //        #READ_METHOD(#READER, #OPT, #ARGS).map(#map)
    //    },
    //};

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
        Ok(())
    }
}
