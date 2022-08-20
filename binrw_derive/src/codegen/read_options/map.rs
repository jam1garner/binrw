#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::Input;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::Ident;

use super::{get_assertions, get_map_err, PreludeGenerator};

pub(crate) fn generate_map(input: &Input, name: Option<&Ident>, map: &TokenStream) -> TokenStream {
    let verify_map = verify_map(input, map.span());
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
        #verify_map
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
    let verify_map = verify_map(input, map.span());
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
        #verify_map
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

fn verify_map(input: &Input, span: Span) -> Option<TokenStream> {
    let has_field_attr = match input {
        Input::Struct(input) => input.fields.iter().any(|field| !field.has_no_attrs()),
        Input::Enum(input) => input.variants.iter().any(|variant| !variant.has_no_attrs()),
        _ => false,
    };

    // TODO: Errors are only supposed to be emitted by the parser
    // if has_field_attr {
    //     Some(quote_spanned! {
    //         span => compile_error!("Field-level attributes cannot be used with top-level map");
    //     })
    // } else {
    None
    // }
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
