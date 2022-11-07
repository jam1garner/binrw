use super::PreludeGenerator;
use crate::binrw::{
    codegen::{
        get_assertions, get_map_err,
        sanitization::{ARGS, OPT, POS, READER, READ_METHOD},
    },
    parser::Input,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident};

pub(crate) fn generate_map(input: &Input, name: Option<&Ident>, map: &TokenStream) -> TokenStream {
    let prelude = PreludeGenerator::new(input)
        .add_imports(name)
        .add_endian()
        .add_magic_pre_assertion()
        .finish();

    let destructure_ref = destructure_ref(input);
    let assertions = field_asserts(input).chain(get_assertions(input.assertions()));
    let reader_var = input.stream_ident_or(READER);

    // TODO: replace args with top-level arguments and only
    // use `()` as a default
    quote! {
        #prelude

        #READ_METHOD(#reader_var, #OPT, ())
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
    let map_err = get_map_err(POS, map.span());
    let prelude = PreludeGenerator::new(input)
        .add_imports(name)
        .add_endian()
        .add_magic_pre_assertion()
        .finish();

    let destructure_ref = destructure_ref(input);
    let assertions = field_asserts(input).chain(get_assertions(input.assertions()));
    let reader_var = input.stream_ident_or(READER);

    // TODO: replace args with top-level arguments and only
    // use `()` as a default
    quote! {
        #prelude

        #READ_METHOD(#reader_var, #OPT, #ARGS).and_then(|value| {
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

fn field_asserts(input: &Input) -> impl Iterator<Item = TokenStream> + '_ {
    match input {
        Input::Struct(input) => either::Left(
            input
                .fields
                .iter()
                .flat_map(|field| get_assertions(&field.assertions)),
        ),
        _ => either::Right(core::iter::empty()),
    }
}
