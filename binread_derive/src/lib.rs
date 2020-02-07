extern crate proc_macro;

use proc_macro::TokenStream;
use quote::*;
use syn::*;
use syn::spanned::Spanned;

mod attributes;
use attributes::{Attributes, parse_attr_setting_group, AttrSetting, SpanError, AttrResult};

mod actions;
use actions::{Action, WriteInstructions};

mod binwrite_endian;

use proc_macro2::{TokenStream as TokenStream2, Span};
use std::result::Result;

macro_rules! macro_error {
    ($name:expr, $item:expr, $message:expr) => {
        {
            let _message = $message;
            let _name = $name;
            let compiler_error = quote_spanned!{$item =>
                compile_error!(#_message);
            };
            return quote!{
                impl ::binwrite::BinWrite for #_name {
                    fn write_options<W: ::std::io::Write>(&self, writer: &mut W, options: &::binwrite::WriterOption) -> ::std::io::Result<()> {
                        Err(::std::io::Error::from(::std::io::ErrorKind::InvalidData))
                    }
                }

                #compiler_error
            }.into()
        }
    }
}

fn parse_attr_setting_group_pair_span(attr: &Attribute) -> Result<Vec<(Span, AttrSetting)>, SpanError> {
    parse_attr_setting_group(attr.tokens.clone())
        .map(|attr_settings|
             attr_settings.iter()
                .map(|attr_setting: &AttrSetting| (attr.span(), attr_setting.clone()))
                .collect()
        )
}

fn some_func(id: Ident, attrs: Vec<AttrSetting>) -> Option<TokenStream2> {
    let WriteInstructions(action, writer_option, gen_options)
        = WriteInstructions::try_from(&attrs)?;

    let writer_option = writer_option_to_tokens(&writer_option);

    let function = match action {
        Action::Default => {
            quote!{::binwrite::BinWrite::write_options}
        }
        Action::CutomerWriter(write_func) => {
            write_func
        }
    };

    let align_before =
        gen_options.align_before
                .map(usize::from)
                .map(gen_align_code);
    let align_after =
        gen_options.align_after
                .map(usize::from)
                .map(gen_align_code);

    let pad_before =
        gen_options.pad_before
                .map(usize::from)
                .map(gen_pad_code); 
    let pad_after =
        gen_options.pad_after
                .map(usize::from)
                .map(gen_pad_code); 

    let value =
        match gen_options.preprocessor {
            Some(preprocessor) => {
                quote!{
                    &((#preprocessor)(&self.#id))
                }
            }
            None => {
                quote!{
                    &self.#id
                }
            }
        };

    let write_value =
        match gen_options.postprocessor {
            Some(postprocessor) => {
                quote!{
                    {
                        let mut _cursor = ::std::io::Cursor::new(::std::vec::Vec::<u8>::new());
                        #function(
                            #value,
                            &mut _cursor,
                            &options
                        )?;

                        ::binwrite::BinWrite::write_options(
                            &(#postprocessor)(_cursor.into_inner()),
                            writer,
                            &options
                        )?;
                    }
                }
            }
            None => {
                quote!{
                    #function(
                        #value,
                        writer,
                        &options
                    )?;
                }
            }
        };

    Some(quote!{
        {
            #align_before
            #pad_before
            #writer_option
            #write_value
            #pad_after
            #align_after
        }
    })
}

#[proc_macro_derive(BinWrite, attributes(binwrite))]
pub fn derive_binwrite(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = input.ident.clone();

    let global_attrs = 
        match attributes::filter_single_attrs(&input.attrs)
            .map(|attrs: Vec<Attribute>|{
            attrs.iter()
                .map(parse_attr_setting_group_pair_span)
                .collect::<Result<Vec<_>, SpanError>>()?
                .into_iter()
                .flatten()
                .map(|(span, attr)| -> Result<TokenStream2, SpanError> {
                    match attr {
                        AttrSetting::Endian(endian) => {
                            let endian: String = (&endian).into();
                            let endian = format_ident!("{}", endian);
                            Ok(quote!{
                                options.endian = ::binwrite::Endian::#endian;
                            })
                        }
                        _ => {
                            Err(SpanError::new(
                                span,
                                "Propety not supported at struct level".into()
                            ))
                        }
                    }
                })
                .collect::<Result<Vec<TokenStream2>, SpanError>>()
                .map(|tokens| quote!{
                    let mut options = options.clone();
                    #(#tokens);*
                })
            })
        .transpose() {
            Ok(a) => a,
            Err(SpanError {
                span, error
            }) => {
                macro_error!(
                    name,
                    span,
                    error
                );
            }
        };

    let fields = match &input.data {
        Data::Struct(data_struct) => {
            data_struct.fields.clone()
        }
        Data::Enum(data_enum) => {
            macro_error!(name, data_enum.enum_token.span, "Derive(BinWrite) for enums not supported");
        }
        Data::Union(data_union) => {
            macro_error!(name, data_union.union_token.span, "Derive(BinWrite) for unions not supported");
        }
    };

    let attrs = Attributes::from_fields(fields);
    let instructions = attrs
                            .map(|attr_result: AttrResult| -> Result<Option<TokenStream2>, SpanError> {
                                let (id, attrs) = attr_result?;
                                Ok(some_func(id, attrs))
                            })
                            .collect::<Result<Vec<Option<TokenStream2>>, SpanError>>();

    let instructions: Vec<_> = match instructions {
        Ok(instructions) => instructions.into_iter().flatten().collect(),
        Err(SpanError{
            span, error
        }) => {
            macro_error!(
                name,
                span,
                error
            );
        }
    };

    let instructions = quote!{#(#instructions)*};

    TokenStream::from(quote! {
        impl ::binwrite::BinWrite for #name {
            fn write_options<W: ::std::io::Write>(&self, writer: &mut W, options: &::binwrite::WriterOption) -> ::std::io::Result<()> {
                let mut _writer = ::binwrite::write_track::WriteTrack::new(writer);
                let writer = &mut _writer;

                #global_attrs

                #instructions

                Ok(())
            }
        }
    })
}

fn gen_align_code(padding: usize) -> TokenStream2 {
    quote!{
        let current = ::std::io::Seek::seek(
            writer,
            ::std::io::SeekFrom::Current(0)
        )? as usize;
        ::binwrite::BinWrite::write_options(
            &vec![0u8; (#padding - (current % #padding)) % #padding][..],
            writer,
            &options
        )?;
    }
}

fn gen_pad_code(padding: usize) -> TokenStream2 {
    quote!{
        {
            ::binwrite::BinWrite::write_options(
                &vec![0u8; #padding][..],
                writer,
                &options
            )?;
        }
    }
}

fn writer_option_to_tokens(writer_option: &actions::OptionalWriterOption) -> Option<TokenStream2> {
    let endian: String = (&writer_option.endian?).into();
    let endian = format_ident!("{}", endian);

    Some(quote!{
        let options = ::binwrite::writer_option_new! {
            endian: ::binwrite::Endian::#endian,
        };
    })
}
