use proc_macro2::{Ident, TokenStream};
use crate::parser::{CondEndian, write::{Input, UnitOnlyEnum, UnitEnumField}};
use quote::quote;

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;

pub(crate) fn generate_unit_enum(
    input: &Input,
    name: Option<&Ident>,
    en: &UnitOnlyEnum
) -> TokenStream {
    match &en.repr {
        Some(repr) => generate_unit_enum_repr(repr, en, &en.fields),
        None => generate_unit_enum_magic(input, name, en, &en.fields),
    }
}

fn specify_endian(endian: &CondEndian) -> Option<TokenStream> {
    match endian {
        CondEndian::Inherited => None,
        CondEndian::Fixed(endian) => Some({
            let endian = endian.as_binrw_endian();
            quote! {
                .clone().with_endian(#endian)
            }
        }),
        CondEndian::Cond(endian, cond) => Some({
            let else_endian = endian.flipped().as_binrw_endian();
            let endian = endian.as_binrw_endian();
            quote! {
                .clone().with_endian(if #cond { #endian } else { #else_endian })
            }
        }),
    }
}

fn generate_unit_enum_repr(
    repr: &TokenStream,
    en: &UnitOnlyEnum,
    variants: &[UnitEnumField],
) -> TokenStream {
    let branches = variants.iter().map(|variant| {
        let name = &variant.ident;
        quote! {
            Self::#name => Self::#name
        }
    });

    let specify_endian = specify_endian(&en.endian);

    quote! {
        #WRITE_METHOD (
            &(match self {
                #(#branches),*
            } as #repr),
            #WRITER,
            &#OPT#specify_endian,
            (),
        )?;
    }
}

fn generate_unit_enum_magic(
    _input: &Input,
    _name: Option<&Ident>,
    _en: &UnitOnlyEnum,
    _variants: &[UnitEnumField],
) -> TokenStream {
    todo!()
}

//struct UnitEnumGenerator<'a> {
//    input: &'a Input,
//    en: &'a UnitOnlyEnum,
//    name: Option<&'a Ident>,
//}
