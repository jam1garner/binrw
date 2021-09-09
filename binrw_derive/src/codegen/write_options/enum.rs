use crate::parser::write::{Input, UnitEnumField, UnitOnlyEnum};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;

use super::prelude::PreludeGenerator;

pub(crate) fn generate_unit_enum(
    input: &Input,
    name: Option<&Ident>,
    en: &UnitOnlyEnum,
) -> TokenStream {
    let write = match &en.repr {
        Some(repr) => generate_unit_enum_repr(repr, &en.fields),
        None => generate_unit_enum_magic(&en.fields),
    };

    PreludeGenerator::new(write, input, name)
        .prefix_magic(&en.magic)
        .prefix_endian(&en.endian)
        .prefix_imports()
        .finish()
}

fn generate_unit_enum_repr(repr: &TokenStream, variants: &[UnitEnumField]) -> TokenStream {
    let branches = variants.iter().map(|variant| {
        let name = &variant.ident;
        quote! {
            Self::#name => Self::#name
        }
    });

    quote! {
        #WRITE_METHOD (
            &(match self {
                #(#branches),*
            } as #repr),
            #WRITER,
            &#OPT,
            (),
        )?;
    }
}

fn generate_unit_enum_magic(variants: &[UnitEnumField]) -> TokenStream {
    let branches = variants.iter().map(|variant| {
        let name = &variant.ident;
        let magic = variant.magic.as_ref().map(|magic| {
            let magic = magic.match_value();

            quote! {
                #WRITE_METHOD (
                    &#magic,
                    #WRITER,
                    &#OPT,
                    (),
                )?;
            }
        });

        quote! {
            Self::#name => {
                #magic
            }
        }
    });

    quote! {
        match self {
            #( #branches )*
        }
    }
}

//struct UnitEnumGenerator<'a> {
//    input: &'a Input,
//    en: &'a UnitOnlyEnum,
//    name: Option<&'a Ident>,
//}
