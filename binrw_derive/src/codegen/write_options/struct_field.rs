use std::ops::Not;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::{CondEndian, FieldMode, Map, PassedArgs, StructField};

pub(crate) fn write_field(field: &StructField) -> TokenStream {
    StructFieldGenerator::new(field)
        .write_field()
        .wrap_padding()
        .prefix_args()
        .prefix_write_fn()
        .prefix_map_fn()
        .prefix_magic()
        .prefix_assertions()
        .finish()
}

struct StructFieldGenerator<'input> {
    field: &'input StructField,
    out: TokenStream,
}

impl<'a> StructFieldGenerator<'a> {
    fn new(field: &'a StructField) -> Self {
        Self {
            field,
            out: TokenStream::new(),
        }
    }

    fn args_ident(&self) -> Ident {
        make_ident(&self.field.ident, "args")
    }

    fn map_fn_ident(&self) -> Ident {
        make_ident(&self.field.ident, "map_fn")
    }

    fn specify_endian(&self) -> Option<TokenStream> {
        match &self.field.endian {
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

    fn prefix_assertions(mut self) -> Self {
        let assertions = super::get_assertions(&self.field.assertions);

        let out = self.out;
        self.out = quote! {
            #(#assertions)*
            #out
        };

        self
    }

    fn prefix_write_fn(mut self) -> Self {
        if !self.field.is_written() {
            return self;
        }

        let write_fn = match &self.field.read_mode {
            FieldMode::Normal | FieldMode::Calc(_) => quote! { #WRITE_METHOD },
            FieldMode::Function(write_fn) => write_fn.clone(),
            FieldMode::Default => unreachable!("Ignored fields are not written"),
        };

        let write_fn = if self.field.map.is_some() {
            let map_fn = self.map_fn_ident();
            if self.field.map.is_try() {
                quote! { #WRITE_FN_TRY_MAP_OUTPUT_TYPE_HINT(&#map_fn, #write_fn) }
            } else {
                quote! { #WRITE_FN_MAP_OUTPUT_TYPE_HINT(&#map_fn, #write_fn) }
            }
        } else {
            quote! { #WRITE_FN_TYPE_HINT(#write_fn) }
        };

        let out = &self.out;
        self.out = quote! {
            let #WRITE_FUNCTION = #write_fn;
            #out
        };

        self
    }

    fn field_mapping(&self) -> Option<TokenStream> {
        match &self.field.map {
            Map::Try(map_fn) | Map::Map(map_fn) => Some(quote! { (#map_fn) }),
            Map::Repr(ty) => Some(quote! { (<#ty as core::convert::TryFrom<_>>::try_from) }),
            Map::None => None,
        }
    }

    fn prefix_map_fn(mut self) -> Self {
        let map_fn = self.field_mapping().map(|map_fn| {
            let map_fn_ident = self.map_fn_ident();

            let ty = &self.field.ty;
            let ty_ref = self.field.generated_value().not().then(|| quote! { & });
            quote! {
                let #map_fn_ident = #WRITE_MAP_INPUT_TYPE_HINT::<#ty_ref #ty, _, _>(#map_fn);
            }
        });

        let out = self.out;
        self.out = quote! {
            #map_fn
            #out
        };

        self
    }

    fn write_field(mut self) -> Self {
        let name = &self.field.ident;
        let args = self.args_ident();
        let specify_endian = self.specify_endian();

        let initialize = match &self.field.read_mode {
            FieldMode::Calc(expr) => Some({
                let ty = &self.field.ty;
                quote! {
                    let #name: #ty = #expr;
                }
            }),
            // If ignored, just skip this now
            FieldMode::Default => return self,
            _ => None,
        };

        let map_fn = self.field.map.is_some().then(|| self.map_fn_ident());
        let map_try = self.field.map.is_try().then(|| {
            let map_err = super::get_map_err(SAVED_POSITION);
            quote! { #map_err? }
        });

        let store_position = quote! {
            let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#WRITER)?;
        };

        self.out = quote! {
            #initialize

            #WRITE_FUNCTION (
                { #store_position &(#map_fn (#name) #map_try) },
                #WRITER,
                &#OPT #specify_endian,
                #args
            )?;
        };

        self
    }

    fn pad_before(&self) -> TokenStream {
        let seek_before = self.field.seek_before.as_ref().map(|seek| {
            quote! {
                #SEEK_TRAIT::seek(
                    #WRITER,
                    #seek,
                )?;
            }
        });
        let pad_before = self.field.pad_before.as_ref().map(|padding| {
            quote! {
                #WRITE_ZEROES(#WRITER, (#padding) as u64)?;
            }
        });
        let align_before = self.field.align_before.as_ref().map(|alignment| {
            quote! {{
                let pos = #SEEK_TRAIT::seek(#WRITER, #SEEK_FROM::Current(0))?;
                let align = ((#alignment) as u64);
                let rem = pos % align;
                if rem != 0 {
                    #WRITE_ZEROES(#WRITER, align - rem)?;
                }
            }}
        });
        let pad_size_to_before = self.field.pad_size_to.as_ref().map(|_| {
            quote! {
                let #BEFORE_POS = #SEEK_TRAIT::seek(#WRITER, #SEEK_FROM::Current(0))?;
            }
        });
        let store_position = self.field.restore_position.map(|_| {
            quote! {
                let #SAVED_POSITION = #SEEK_TRAIT::seek(#WRITER, #SEEK_FROM::Current(0))?;
            }
        });

        quote! {
            #store_position
            #seek_before
            #pad_before
            #align_before
            #pad_size_to_before
        }
    }

    fn pad_after(&self) -> TokenStream {
        let pad_size_to = self.field.pad_size_to.as_ref().map(|size| {
            quote! {{
                let pad_to_size = (#size) as u64;
                let after_pos = #SEEK_TRAIT::seek(#WRITER, #SEEK_FROM::Current(0))?;
                if let Some(size) = after_pos.checked_sub(#BEFORE_POS) {
                    if let Some(padding) = pad_to_size.checked_sub(size) {
                        #WRITE_ZEROES(#WRITER, padding)?;
                    }
                }
            }}
        });
        let pad_after = self.field.pad_after.as_ref().map(|padding| {
            quote! {
                #WRITE_ZEROES(#WRITER, (#padding) as u64)?;
            }
        });
        let align_after = self.field.align_after.as_ref().map(|alignment| {
            quote! {{
                let pos = #SEEK_TRAIT::seek(#WRITER, #SEEK_FROM::Current(0))?;
                let align = ((#alignment) as u64);
                let rem = pos % align;
                if rem != 0 {
                    #WRITE_ZEROES(#WRITER, align - rem)?;
                }
            }}
        });
        let restore_position = self.field.restore_position.map(|_| {
            quote! {
                #SEEK_TRAIT::seek(#WRITER, #SEEK_FROM::Start(#SAVED_POSITION))?;
            }
        });

        quote! {
            #pad_size_to
            #pad_after
            #align_after
            #restore_position
        }
    }

    fn wrap_padding(mut self) -> Self {
        let out = &self.out;

        let pad_before = self.pad_before();
        let pad_after = self.pad_after();
        self.out = quote! {
            #pad_before
            #out
            #pad_after
        };

        self
    }

    fn prefix_args(mut self) -> Self {
        if !self.field.is_written() {
            return self;
        }

        let args = self.args_ident();

        let args_val = if let Some(args) = get_passed_args(self.field) {
            args
        } else {
            quote! { () }
        };

        let map_fn = self.map_fn_ident();
        let out = &self.out;
        self.out = match self.field.read_mode {
            FieldMode::Normal => match &self.field.map {
                Map::Map(_) => quote! {
                    let #args = #WRITE_MAP_ARGS_TYPE_HINT(&#map_fn, #args_val);
                    #out
                },
                Map::Try(_) | Map::Repr(_) => quote! {
                    let #args = #WRITE_TRY_MAP_ARGS_TYPE_HINT(&#map_fn, #args_val);
                    #out
                },
                Map::None => {
                    let ty = &self.field.ty;
                    quote! {
                        let #args: <#ty as #BINWRITE_TRAIT>::Args = #args_val;
                        #out
                    }
                }
            },
            FieldMode::Calc(_) => quote! {
                let #args = ();
                #out
            },
            FieldMode::Function(_) => {
                let ty = &self.field.ty;
                quote! {
                    let #args = #WRITE_WITH_ARGS_TYPE_HINT::<#ty, W, _, _>(
                        #WRITE_FUNCTION, #args_val
                    );
                    #out
                }
            }
            FieldMode::Default => unreachable!("Ignored fields are not written"),
        };

        self
    }

    fn prefix_magic(mut self) -> Self {
        if let Some(magic) = &self.field.magic {
            let magic = magic.match_value();
            let specify_endian = self.specify_endian();
            let out = self.out;
            self.out = quote! {
                #WRITE_METHOD (
                    &#magic,
                    #WRITER,
                    &#OPT #specify_endian,
                    ()
                )?;

                #out
            };
        }

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }
}

fn get_passed_args(field: &StructField) -> Option<TokenStream> {
    let args = &field.args;
    match args {
        PassedArgs::Named(fields) => Some(if let Some(count) = &field.count {
            quote! {
                #ARGS_MACRO! { count: ((#count) as usize) #(, #fields)* }
            }
        } else {
            quote! {
                #ARGS_MACRO! { #(#fields),* }
            }
        }),
        PassedArgs::List(list) => Some(quote! { (#(#list,)*) }),
        PassedArgs::Tuple(tuple) => Some(tuple.as_ref().clone()),
        PassedArgs::None => field
            .count
            .as_ref()
            .map(|count| quote! { #ARGS_MACRO! { count: ((#count) as usize) }}),
    }
}
