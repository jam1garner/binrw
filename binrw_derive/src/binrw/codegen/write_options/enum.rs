use super::{prelude::PreludeGenerator, r#struct::StructGenerator};
use crate::binrw::{
    codegen::sanitization::{OPT, WRITER, WRITE_METHOD},
    parser::{Enum, EnumVariant, Input, UnitEnumField, UnitOnlyEnum},
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub(crate) fn generate_unit_enum(
    input: &Input,
    name: Option<&Ident>,
    en: &UnitOnlyEnum,
) -> TokenStream {
    let writer_var = input.stream_ident_or(WRITER);
    let write = match en.map.as_repr() {
        Some(repr) => generate_unit_enum_repr(&writer_var, repr, &en.fields),
        None => generate_unit_enum_magic(&writer_var, &en.fields),
    };

    PreludeGenerator::new(write, input, name, &writer_var)
        .prefix_map_stream()
        .prefix_magic(&en.magic)
        .prefix_assertions()
        .prefix_endian(&en.endian)
        .prefix_imports()
        .finish()
}

pub(crate) fn generate_data_enum(input: &Input, name: Option<&Ident>, en: &Enum) -> TokenStream {
    EnumGenerator::new(input, name, en, input.stream_ident_or(WRITER))
        .write_variants()
        .prefix_prelude()
        .finish()
}

struct EnumGenerator<'a> {
    en: &'a Enum,
    input: &'a Input,
    name: Option<&'a Ident>,
    writer_var: TokenStream,
    out: TokenStream,
}

impl<'a> EnumGenerator<'a> {
    fn new(
        input: &'a Input,
        name: Option<&'a Ident>,
        en: &'a Enum,
        writer_var: TokenStream,
    ) -> Self {
        Self {
            input,
            name,
            en,
            writer_var,
            out: TokenStream::new(),
        }
    }

    fn write_variants(mut self) -> Self {
        let variants = self.en.variants.iter().map(|variant| {
            let name = variant.ident();
            let fields = match variant {
                EnumVariant::Variant { options, .. } => Some(options.fields_pattern()),
                EnumVariant::Unit(_) => None,
            };

            let writer_var = &self.writer_var;
            let writing = match variant {
                EnumVariant::Variant { options, .. } => {
                    let input = Input::Struct(variant.clone().into());

                    StructGenerator::new(&input, options, None, &self.writer_var)
                        .write_fields()
                        .prefix_prelude()
                        .finish()
                }
                EnumVariant::Unit(variant) => variant
                    .magic
                    .as_ref()
                    .map(|magic| {
                        let magic = magic.match_value();
                        quote! {
                            #WRITE_METHOD (
                                &#magic,
                                #writer_var,
                                #OPT,
                                ()
                            )?;
                        }
                    })
                    .unwrap_or_default(),
            };

            quote! {
                Self::#name #fields => {
                    #writing
                }
            }
        });

        self.out = quote! {
            match self {
                #( #variants )*
            }
        };

        self
    }

    fn prefix_prelude(mut self) -> Self {
        let out = self.out;

        self.out = PreludeGenerator::new(out, self.input, self.name, &self.writer_var)
            .prefix_map_stream()
            .prefix_magic(&self.en.magic)
            .prefix_assertions()
            .prefix_endian(&self.en.endian)
            .prefix_imports()
            .finish();

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }
}

fn generate_unit_enum_repr(
    writer_var: &TokenStream,
    repr: &TokenStream,
    variants: &[UnitEnumField],
) -> TokenStream {
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
            #writer_var,
            #OPT,
            (),
        )?;
    }
}

fn generate_unit_enum_magic(writer_var: &TokenStream, variants: &[UnitEnumField]) -> TokenStream {
    let branches = variants.iter().map(|variant| {
        let name = &variant.ident;
        let magic = variant.magic.as_ref().map(|magic| {
            let magic = magic.match_value();

            quote! {
                #WRITE_METHOD (
                    &#magic,
                    #writer_var,
                    #OPT,
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
