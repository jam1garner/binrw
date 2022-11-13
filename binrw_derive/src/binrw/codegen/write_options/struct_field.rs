use crate::binrw::{
    codegen::{
        get_assertions, get_endian, get_map_err, get_passed_args, get_try_calc,
        sanitization::{
            make_ident, BEFORE_POS, BINWRITE_TRAIT, POS, SAVED_POSITION, SEEK_FROM, SEEK_TRAIT,
            WRITER, WRITE_ARGS_TYPE_HINT, WRITE_FN_MAP_OUTPUT_TYPE_HINT,
            WRITE_FN_TRY_MAP_OUTPUT_TYPE_HINT, WRITE_FN_TYPE_HINT, WRITE_FUNCTION,
            WRITE_MAP_ARGS_TYPE_HINT, WRITE_MAP_INPUT_TYPE_HINT, WRITE_METHOD,
            WRITE_TRY_MAP_ARGS_TYPE_HINT, WRITE_ZEROES,
        },
    },
    parser::{FieldMode, Map, StructField},
};
use core::ops::Not;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn write_field(field: &StructField) -> TokenStream {
    StructFieldGenerator::new(field)
        .write_field()
        .wrap_padding()
        .prefix_args()
        .prefix_write_fn()
        .prefix_map_fn()
        .prefix_magic()
        .wrap_condition()
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

    fn prefix_assertions(mut self) -> Self {
        let assertions = get_assertions(&self.field.assertions);

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

        let write_fn = match &self.field.field_mode {
            FieldMode::Normal | FieldMode::Calc(_) | FieldMode::TryCalc(_) => {
                quote! { #WRITE_METHOD }
            }
            FieldMode::Function(write_fn) => write_fn.clone(),
            FieldMode::Default => unreachable!("Ignored fields are not written"),
        };

        let write_fn = if self.field.map.is_some() {
            let map_fn = map_fn_ident(&self.field.ident);
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

    fn prefix_map_fn(mut self) -> Self {
        let map_fn = field_mapping(&self.field.map).map(|map_fn| {
            let fn_ident = map_fn_ident(&self.field.ident);

            let ty = &self.field.ty;
            let ty_ref = self.field.generated_value().not().then(|| quote! { & });
            quote! {
                let #fn_ident = #WRITE_MAP_INPUT_TYPE_HINT::<#ty_ref #ty, _, _>(#map_fn);
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
        let args = args_ident(name);
        let endian = get_endian(&self.field.endian);

        let initialize = match &self.field.field_mode {
            FieldMode::Calc(expr) => Some({
                let ty = &self.field.ty;
                quote! {
                    let #name: #ty = #expr;
                }
            }),
            FieldMode::TryCalc(expr) => Some({
                let ty = &self.field.ty;
                let expr = get_try_calc(POS, &self.field.ty, expr);
                quote! {
                    let #name: #ty = #expr;
                }
            }),
            // If ignored, just skip this now
            FieldMode::Default => return self,
            _ => None,
        };

        let map_fn = self.field.map.is_some().then(|| map_fn_ident(name));
        let map_try = self.field.map.is_try().then(|| {
            let map_err = get_map_err(SAVED_POSITION, name.span());
            quote! { #map_err? }
        });

        let store_position = quote! {
            let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#WRITER)?;
        };

        let name = self
            .field
            .if_cond
            .as_ref()
            .and_then(|cond| {
                cond.alternate.as_ref().map(|alternate| {
                    let condition = &cond.condition;
                    quote! {
                        if #condition {
                            #name
                        } else {
                            &#alternate
                        }
                    }
                })
            })
            .unwrap_or_else(|| quote::ToTokens::to_token_stream(name));

        self.out = quote! {
            #initialize

            #WRITE_FUNCTION (
                { #store_position &(#map_fn (#name) #map_try) },
                #WRITER,
                #endian,
                #args
            )?;
        };

        self
    }

    fn wrap_condition(mut self) -> Self {
        if let Some(cond) = &self.field.if_cond {
            if cond.alternate.is_none() {
                let condition = &cond.condition;
                let consequent = self.out;
                self.out = quote! {
                    if #condition {
                        #consequent
                    }
                };
            }
        }

        self
    }

    fn wrap_padding(mut self) -> Self {
        let out = &self.out;

        let pad_before = pad_before(self.field);
        let pad_after = pad_after(self.field);
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

        let args = args_ident(&self.field.ident);

        let args_val = if let Some(args) = get_passed_args(self.field) {
            args
        } else {
            quote! { () }
        };

        let map_fn = map_fn_ident(&self.field.ident);
        let out = self.out;
        self.out = match &self.field.field_mode {
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
                        let #args: <#ty as #BINWRITE_TRAIT>::Args<'_> = #args_val;
                        #out
                    }
                }
            },
            FieldMode::Calc(_) | FieldMode::TryCalc(_) => quote! {
                let #args = ();
                #out
            },
            FieldMode::Function(_) => {
                let ty = &self.field.ty;
                quote! {
                    let #args = #WRITE_ARGS_TYPE_HINT::<#ty, W, _, _>(
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
            let endian = get_endian(&self.field.endian);
            let out = self.out;
            self.out = quote! {
                #WRITE_METHOD (
                    &#magic,
                    #WRITER,
                    #endian,
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

fn args_ident(ident: &Ident) -> Ident {
    make_ident(ident, "args")
}

fn field_mapping(map: &Map) -> Option<TokenStream> {
    match map {
        Map::Try(map_fn) | Map::Map(map_fn) => Some(quote! { (#map_fn) }),
        Map::Repr(ty) => Some(quote! { (<#ty as core::convert::TryFrom<_>>::try_from) }),
        Map::None => None,
    }
}

fn map_fn_ident(ident: &Ident) -> Ident {
    make_ident(ident, "map_fn")
}

fn pad_after(field: &StructField) -> TokenStream {
    let pad_size_to = field.pad_size_to.as_ref().map(|size| {
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
    let pad_after = field.pad_after.as_ref().map(|padding| {
        quote! {
            #WRITE_ZEROES(#WRITER, (#padding) as u64)?;
        }
    });
    let align_after = field.align_after.as_ref().map(|alignment| {
        quote! {{
            let pos = #SEEK_TRAIT::seek(#WRITER, #SEEK_FROM::Current(0))?;
            let align = ((#alignment) as u64);
            let rem = pos % align;
            if rem != 0 {
                #WRITE_ZEROES(#WRITER, align - rem)?;
            }
        }}
    });
    let restore_position = field.restore_position.map(|_| {
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

fn pad_before(field: &StructField) -> TokenStream {
    let seek_before = field.seek_before.as_ref().map(|seek| {
        quote! {
            #SEEK_TRAIT::seek(
                #WRITER,
                #seek,
            )?;
        }
    });
    let pad_before = field.pad_before.as_ref().map(|padding| {
        quote! {
            #WRITE_ZEROES(#WRITER, (#padding) as u64)?;
        }
    });
    let align_before = field.align_before.as_ref().map(|alignment| {
        quote! {{
            let pos = #SEEK_TRAIT::seek(#WRITER, #SEEK_FROM::Current(0))?;
            let align = ((#alignment) as u64);
            let rem = pos % align;
            if rem != 0 {
                #WRITE_ZEROES(#WRITER, align - rem)?;
            }
        }}
    });
    let pad_size_to_before = field.pad_size_to.as_ref().map(|_| {
        quote! {
            let #BEFORE_POS = #SEEK_TRAIT::seek(#WRITER, #SEEK_FROM::Current(0))?;
        }
    });
    let store_position = field.restore_position.map(|_| {
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
