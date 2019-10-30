extern crate proc_macro;

use proc_macro::TokenStream;
use quote::*;
use syn::*;

mod attributes;
use attributes::Attributes;

mod actions;
use actions::{Action, WriteInstructions};

use binwrite::WriterOption;
use proc_macro2::TokenStream as TokenStream2;

#[proc_macro_derive(BinWrite, attributes(binwrite))]
pub fn derive_binwrite(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = input.ident.clone();

    let fields = match &input.data {
        Data::Struct(data_struct) => {
            data_struct.fields.clone()
        }
        _ => {
            unimplemented!()
        }
    };

    let attrs = Attributes::from_fields(fields);
    let instructions = attrs
                            .map(|(id, attrs)|{
                                let WriteInstructions(action, writer_option, gen_options)
                                        = WriteInstructions::from(&attrs);

                                let writer_option = writer_option_to_tokens(&writer_option);

                                let function = match action {
                                    Action::Default => {
                                        quote!{::binwrite::BinWrite::write_options}
                                    }
                                    Action::CutomerWriter(write_func) => {
                                        write_func.into()
                                    }
                                };

                                let pad_before =
                                    gen_options.pad_before
                                            .map(usize::from)
                                            .map(gen_pad_code); 
                                let pad_after =
                                    gen_options.pad_after
                                            .map(usize::from)
                                            .map(gen_pad_code); 

                                quote!{
                                    #pad_before
                                    #function(
                                        &self.#id,
                                        writer,
                                        &#writer_option
                                    )?;
                                    #pad_after
                                }
                            })
                            .collect::<Vec<TokenStream2>>();

    let instructions = quote!{#(#instructions);*};

    TokenStream::from(quote! {
        impl ::binwrite::BinWrite for #name {
            fn write_options<W: ::std::io::Write>(&self, writer: &mut W, options: &::binwrite::WriterOption) -> ::std::io::Result<()> {
                let mut _writer = binwrite::write_track::WriteTrack::new(writer);
                let writer = &mut _writer;

                #instructions

                Ok(())
            }
        }
    })
}

fn gen_pad_code(padding: usize) -> TokenStream2 {
    quote!{
        {
            let current = ::std::io::Seek::seek(
                writer,
                ::std::io::SeekFrom::Current(0)
            )? as usize;
            ::binwrite::BinWrite::write_options(
                &vec![0u8; (#padding - (current % #padding)) % #padding][..],
                writer,
                options
            )?;
        }
    }
}

fn writer_option_to_tokens(writer_option: &WriterOption) -> TokenStream2 {
    let endian: String = writer_option.endian.into();
    let endian = format_ident!("{}", endian);

    quote!{
        ::binwrite::WriterOption {
            endian: ::binwrite::Endian::#endian,
            others: Vec::new(),
        }
    }
}
