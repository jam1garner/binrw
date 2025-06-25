use crate::{
    binrw::{
        codegen::{
            get_assertions, get_endian, get_map_err, get_passed_args, get_try_calc,
            sanitization::{
                make_ident, BEFORE_POS, BINWRITE_TRAIT, MAP_WRITER_TYPE_HINT, POS,
                REQUIRED_ARG_TRAIT, SAVED_POSITION, SEEK_FROM, SEEK_TRAIT, WRITE_ARGS_TYPE_HINT,
                WRITE_FN_MAP_OUTPUT_TYPE_HINT, WRITE_FN_TRY_MAP_OUTPUT_TYPE_HINT,
                WRITE_FN_TYPE_HINT, WRITE_FUNCTION, WRITE_MAP_ARGS_TYPE_HINT,
                WRITE_MAP_INPUT_TYPE_HINT, WRITE_METHOD, WRITE_TRY_MAP_ARGS_TYPE_HINT,
                WRITE_ZEROES,
            },
        },
        parser::{FieldMode, Map, StructField},
    },
    util::quote_spanned_any,
};
use alloc::borrow::Cow;
use core::ops::Not;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Ident};

pub(crate) fn write_field(writer_var: &TokenStream, field: &StructField) -> TokenStream {
    StructFieldGenerator::new(field, writer_var)
        .write_field()
        .wrap_map_stream()
        .prefix_map_value()
        .prefix_calc_value()
        .wrap_padding()
        .prefix_magic()
        .wrap_condition()
        .prefix_assertions()
        .prefix_args()
        .prefix_write_function()
        .prefix_map_function()
        .finish()
}

struct StructFieldGenerator<'input> {
    field: &'input StructField,
    outer_writer_var: &'input TokenStream,
    writer_var: Cow<'input, TokenStream>,
    out: TokenStream,
}

impl<'a> StructFieldGenerator<'a> {
    fn new(field: &'a StructField, outer_writer_var: &'a TokenStream) -> Self {
        Self {
            field,
            outer_writer_var,
            writer_var: if field.map_stream.is_some() {
                Cow::Owned(make_ident(&field.ident, "reader").into_token_stream())
            } else {
                Cow::Borrowed(outer_writer_var)
            },
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

    fn wrap_map_stream(mut self) -> Self {
        if let Some(map_stream) = &self.field.map_stream {
            let rest = self.out;
            let writer_var = &self.writer_var;
            let outer_writer_var = self.outer_writer_var;
            self.out = quote_spanned_any! { map_stream.span()=> {
                let #writer_var = &mut #MAP_WRITER_TYPE_HINT::<W, _, _>(#map_stream)(#outer_writer_var);
                #rest
            }};
        }

        self
    }

    fn prefix_write_function(mut self) -> Self {
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
            let map_fn = map_func_ident(&self.field.ident);
            if self.field.map.is_try() {
                quote! { #WRITE_FN_TRY_MAP_OUTPUT_TYPE_HINT(&#map_fn, #write_fn) }
            } else {
                quote! { #WRITE_FN_MAP_OUTPUT_TYPE_HINT(&#map_fn, #write_fn) }
            }
        } else {
            let ty = &self.field.ty;
            quote! { #WRITE_FN_TYPE_HINT::<#ty, _, _, _>(#write_fn) }
        };

        let out = self.out;
        self.out = quote! {
            let #WRITE_FUNCTION = #write_fn;
            #out
        };

        self
    }

    fn prefix_map_function(mut self) -> Self {
        let map_func = field_mapping(&self.field.map).map(|map_fn| {
            let map_func = map_func_ident(&self.field.ident);

            let ty = &self.field.ty;
            let ty_ref = self.field.generated_value().not().then(|| quote! { & });
            quote! {
                let #map_func = #WRITE_MAP_INPUT_TYPE_HINT::<#ty_ref #ty, _, _>(#map_fn);
            }
        });

        let out = self.out;
        self.out = quote! {
            #map_func
            #out
        };

        self
    }

    fn prefix_calc_value(mut self) -> Self {
        let name = &self.field.ident;
        let ty = &self.field.ty;
        let expr = match &self.field.field_mode {
            FieldMode::Calc(expr) => expr.clone(),
            FieldMode::TryCalc(expr) => get_try_calc(POS, &self.field.ty, expr),
            _ => return self,
        };

        let rest = self.out;
        self.out = quote! {
            let #name: #ty = #expr;
            #rest
        };

        self
    }

    fn prefix_map_value(mut self) -> Self {
        let name = &self.field.ident;
        let map_func = self.field.map.is_some().then(|| map_func_ident(name));

        self.out = match &self.field.map {
            Map::None => return self,
            Map::Map(_) => {
                let rest = self.out;
                quote! {
                    let #name = #map_func(#name);
                    #rest
                }
            }
            Map::Try(t) | Map::Repr(t) => {
                let rest = self.out;
                let map_err = get_map_err(SAVED_POSITION, t.span());
                let outer_writer_var = self.outer_writer_var;
                quote! {
                    let #name = {
                        let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#outer_writer_var)?;
                        #map_func(#name)#map_err?
                    };
                    #rest
                }
            }
        };

        self
    }

    fn write_field(mut self) -> Self {
        if !self.field.is_written() {
            return self;
        }

        let name = &self.field.ident;
        let args = args_ident(name);
        let endian = get_endian(&self.field.endian);
        let writer_var = &self.writer_var;

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
            .unwrap_or_else(|| quote_spanned! { name.span()=> &#name });

        self.out = quote! {
            #WRITE_FUNCTION(
                #name,
                #writer_var,
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
        let out = self.out;

        let pad_before = pad_before(self.outer_writer_var, self.field);
        let pad_after = pad_after(self.outer_writer_var, self.field);
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

        let args_val = if let Some(args) = get_passed_args(self.field, self.outer_writer_var) {
            args
        } else {
            quote_spanned! { self.field.ty.span() => <_ as #REQUIRED_ARG_TRAIT>::args() }
        };

        let map_fn = map_func_ident(&self.field.ident);
        let out = self.out;
        self.out = match &self.field.field_mode {
            FieldMode::Calc(_) | FieldMode::TryCalc(_) | FieldMode::Normal => match &self.field.map
            {
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
            FieldMode::Function(_) => quote! {
                let #args = #WRITE_ARGS_TYPE_HINT(&#WRITE_FUNCTION, #args_val);
                #out
            },
            FieldMode::Default => unreachable!("Ignored fields are not written"),
        };

        self
    }

    fn prefix_magic(mut self) -> Self {
        if let Some(magic) = &self.field.magic {
            let magic = magic.match_value();
            let endian = get_endian(&self.field.endian);
            let writer_var = self.outer_writer_var;
            let out = self.out;
            self.out = quote! {
                #WRITE_METHOD (
                    &#magic,
                    #writer_var,
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

fn map_func_ident(ident: &Ident) -> Ident {
    make_ident(ident, "map_func")
}

fn pad_after(writer_var: &TokenStream, field: &StructField) -> TokenStream {
    let pad_size_to = field.pad_size_to.as_ref().map(|size| {
        quote! {{
            let pad_to_size = (#size) as u64;
            let after_pos = #SEEK_TRAIT::stream_position(#writer_var)?;
            if let Some(size) = after_pos.checked_sub(#BEFORE_POS) {
                if let Some(padding) = pad_to_size.checked_sub(size) {
                    #WRITE_ZEROES(#writer_var, padding)?;
                }
            }
        }}
    });
    let pad_after = field.pad_after.as_ref().map(|padding| {
        quote! {
            #WRITE_ZEROES(#writer_var, (#padding) as u64)?;
        }
    });
    let align_after = field.align_after.as_ref().map(|alignment| {
        quote! {{
            let pos = #SEEK_TRAIT::stream_position(#writer_var)?;
            let align = ((#alignment) as u64);
            let rem = pos % align;
            if rem != 0 {
                #WRITE_ZEROES(#writer_var, align - rem)?;
            }
        }}
    });
    let restore_position = field.restore_position.map(|()| {
        quote! {
            #SEEK_TRAIT::seek(#writer_var, #SEEK_FROM::Start(#SAVED_POSITION))?;
        }
    });

    quote! {
        #pad_size_to
        #pad_after
        #align_after
        #restore_position
    }
}

fn pad_before(writer_var: &TokenStream, field: &StructField) -> TokenStream {
    let seek_before = field.seek_before.as_ref().map(|seek| {
        quote! {
            #SEEK_TRAIT::seek(
                #writer_var,
                #seek,
            )?;
        }
    });
    let pad_before = field.pad_before.as_ref().map(|padding| {
        quote! {
            #WRITE_ZEROES(#writer_var, (#padding) as u64)?;
        }
    });
    let align_before = field.align_before.as_ref().map(|alignment| {
        quote! {{
            let pos = #SEEK_TRAIT::stream_position(#writer_var)?;
            let align = ((#alignment) as u64);
            let rem = pos % align;
            if rem != 0 {
                #WRITE_ZEROES(#writer_var, align - rem)?;
            }
        }}
    });
    let pad_size_to_before = field.pad_size_to.as_ref().map(|_| {
        quote! {
            let #BEFORE_POS = #SEEK_TRAIT::stream_position(#writer_var)?;
        }
    });
    let store_position = field.restore_position.map(|()| {
        quote! {
            let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#writer_var)?;
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
