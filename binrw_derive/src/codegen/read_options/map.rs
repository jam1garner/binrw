use super::{get_map_err, PreludeGenerator};
use crate::{
    codegen::{
        get_assertions,
        sanitization::{ARGS, OPT, POS, READER, READ_METHOD},
    },
    parser::Input,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn generate_map(input: &Input, name: Option<&Ident>, map: &TokenStream) -> TokenStream {
    let prelude = PreludeGenerator::new(input)
        .add_imports(name)
        .add_options()
        .add_magic_pre_assertion()
        .finish();

    let destructure_ref = destructure_ref(input);
    let assertions = input
        .field_asserts()
        .chain(get_assertions(input.assertions()));

    // TODO: replace args with top-level arguments and only
    // use `()` as a default
    quote! {
        #prelude

        #READ_METHOD(#READER, #OPT, ())
            .map(#map)
                .and_then(|this| {
                    #destructure_ref

                    (|| {
                        #(
                            #assertions
                        )*

                        Ok(())
                    })().map(|_: ()| this)
                })
    }
}

pub(crate) fn generate_try_map(
    input: &Input,
    name: Option<&Ident>,
    map: &TokenStream,
) -> TokenStream {
    let map_err = get_map_err(POS);
    let prelude = PreludeGenerator::new(input)
        .add_imports(name)
        .add_options()
        .add_magic_pre_assertion()
        .finish();

    let destructure_ref = destructure_ref(input);
    let assertions = input
        .field_asserts()
        .chain(get_assertions(input.assertions()));

    // TODO: replace args with top-level arguments and only
    // use `()` as a default
    quote! {
        #prelude

        #READ_METHOD(#READER, #OPT, #ARGS).and_then(|value| {
            (#map)(value)#map_err
        })
        .and_then(|this| {
            #destructure_ref

            (|| {
                #(
                    #assertions
                )*

                Ok(())
            })().map(|_: ()| this)
        })
    }
}

fn destructure_ref(input: &Input) -> Option<TokenStream> {
    match input {
        Input::Struct(input) => {
            let fields = input.fields.iter().map(|field| &field.ident);

            if input.is_tuple() {
                Some(quote! {
                    let Self ( #( ref #fields ),* ) = &this;
                })
            } else {
                Some(quote! {
                    let Self { #( ref #fields ),* } = &this;
                })
            }
        }

        _ => None,
    }
}
