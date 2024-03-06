use super::{get_magic, PreludeGenerator};
#[cfg(feature = "verbose-backtrace")]
use crate::binrw::backtrace::BacktraceFrame;
use crate::binrw::parser::Assert;
use crate::{
    binrw::{
        codegen::{
            get_assertions, get_endian, get_map_err, get_passed_args, get_try_calc,
            sanitization::{
                make_ident, ARGS_TYPE_HINT, BACKTRACE_FRAME, BINREAD_TRAIT, COERCE_FN,
                DBG_EPRINTLN, MAP_ARGS_TYPE_HINT, MAP_READER_TYPE_HINT, OPT, PARSE_FN_TYPE_HINT,
                POS, READER, READ_FUNCTION, READ_METHOD, REQUIRED_ARG_TRAIT, SAVED_POSITION,
                SEEK_FROM, SEEK_TRAIT, TEMP, THIS, WITH_CONTEXT,
            },
        },
        parser::{ErrContext, FieldMode, Input, Map, Struct, StructField},
    },
    util::quote_spanned_any,
};
use alloc::borrow::Cow;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Ident};

pub(super) fn generate_unit_struct(
    input: &Input,
    name: Option<&Ident>,
    variant_ident: Option<&Ident>,
) -> TokenStream {
    let prelude = get_prelude(input, name);
    let return_type = get_return_type(variant_ident);
    quote! {
        #prelude
        Ok(#return_type)
    }
}

pub(super) fn generate_struct(input: &Input, name: Option<&Ident>, st: &Struct) -> TokenStream {
    StructGenerator::new(input, st)
        .read_fields(name, None)
        .initialize_value_with_assertions(None, &[])
        .wrap_pad_size()
        .return_value()
        .finish()
}

pub(super) struct StructGenerator<'input> {
    input: &'input Input,
    st: &'input Struct,
    out: TokenStream,
}

impl<'input> StructGenerator<'input> {
    pub(super) fn new(input: &'input Input, st: &'input Struct) -> Self {
        Self {
            input,
            st,
            out: TokenStream::new(),
        }
    }

    pub(super) fn finish(self) -> TokenStream {
        self.out
    }

    pub(super) fn initialize_value_with_assertions(
        self,
        variant_ident: Option<&Ident>,
        extra_assertions: &[Assert],
    ) -> Self {
        if self.has_self_assertions(extra_assertions) {
            self.init_value(variant_ident)
                .add_assertions(extra_assertions)
        } else {
            self.add_assertions(extra_assertions)
                .init_value(variant_ident)
        }
    }

    fn has_self_assertions(&self, extra_assertions: &[Assert]) -> bool {
        self.st
            .assertions
            .iter()
            .chain(extra_assertions)
            .any(|assert| assert.condition_uses_self)
    }

    fn add_assertions(mut self, extra_assertions: &[Assert]) -> Self {
        let assertions =
            get_assertions(&self.st.assertions).chain(get_assertions(extra_assertions));
        let head = self.out;
        self.out = quote! {
            #head
            #(#assertions)*
        };

        self
    }

    fn wrap_pad_size(mut self) -> Self {
        if let Some(pad) = &self.st.pad_size_to {
            let reader_var = self.input.stream_ident_or(READER);
            let value = self.out;
            self.out = quote! {
                let #POS = #SEEK_TRAIT::stream_position(#reader_var)?;
                #value
                {
                    let pad = (#pad) as i64;
                    let size = (#SEEK_TRAIT::stream_position(#reader_var)? - #POS) as i64;
                    if size < pad {
                        #SEEK_TRAIT::seek(#reader_var, #SEEK_FROM::Current(pad - size))?;
                    }
                }
            };
        }

        self
    }

    pub(super) fn read_fields(mut self, name: Option<&Ident>, variant_name: Option<&str>) -> Self {
        let prelude = get_prelude(self.input, name);
        let read_fields = self
            .st
            .fields
            .iter()
            .map(|field| generate_field(self.input, field, name, variant_name));
        self.out = quote! {
            #prelude
            #(#read_fields)*
        };

        self
    }

    fn init_value(mut self, variant_ident: Option<&Ident>) -> Self {
        let out_names = self.st.iter_permanent_idents();
        let return_type = get_return_type(variant_ident);
        let return_value = if self.st.is_tuple() {
            quote! { #return_type(#(#out_names),*) }
        } else {
            quote! { #return_type { #(#out_names),* } }
        };

        let head = self.out;
        self.out = quote! {
            #head
            let #THIS = #return_value;
        };

        self
    }

    pub(super) fn return_value(mut self) -> Self {
        let head = self.out;

        self.out = quote! {
            #head
            Ok(#THIS)
        };

        self
    }
}

fn generate_field(
    input: &Input,
    field: &StructField,
    name: Option<&Ident>,
    variant_name: Option<&str>,
) -> TokenStream {
    // temp + ignore == just don't bother
    if field.is_temp(false) && matches!(field.field_mode, FieldMode::Default) {
        return TokenStream::new();
    }

    FieldGenerator::new(input, field)
        .read_value()
        .wrap_map_stream()
        .try_conversion(name, variant_name)
        .map_value()
        .wrap_debug()
        .wrap_seek()
        .wrap_condition()
        .assign_to_var()
        .append_assertions()
        .wrap_restore_position()
        .prefix_magic()
        .prefix_args_and_options()
        .prefix_map_function()
        .prefix_read_function()
        .finish()
}

struct FieldGenerator<'field> {
    field: &'field StructField,
    out: TokenStream,
    outer_reader_var: TokenStream,
    reader_var: TokenStream,
    endian_var: TokenStream,
    args_var: Option<Ident>,
}

impl<'field> FieldGenerator<'field> {
    fn new(input: &Input, field: &'field StructField) -> Self {
        let (reader_var, endian_var, args_var) = make_field_vars(input, field);

        Self {
            field,
            out: TokenStream::new(),
            outer_reader_var: input.stream_ident_or(READER),
            reader_var,
            endian_var,
            args_var,
        }
    }

    fn wrap_debug(mut self) -> Self {
        // Unwrapping the proc-macro2 Span is undesirable but necessary until its API
        // is updated to allow retrieving line/column again. Using a separate function
        // to unwrap just to make it clearer what needs to be undone later.
        // <https://github.com/dtolnay/proc-macro2/pull/383>
        #[cfg(all(feature = "verbose-backtrace", nightly, proc_macro))]
        fn start_line(span: proc_macro2::Span) -> usize {
            span.unwrap().start().line()
        }
        #[cfg(not(all(feature = "verbose-backtrace", nightly, proc_macro)))]
        fn start_line(_: proc_macro2::Span) -> usize {
            0
        }

        if self.field.debug.is_some() {
            fn dbg_space(
                name: &'static str,
                at: &TokenStream,
                which: Option<&TokenStream>,
            ) -> Option<TokenStream> {
                which.map(|space| {
                    quote_spanned! {space.span()=> {
                        #DBG_EPRINTLN!(
                            ::core::concat!("[{}:{} | ", #name, " {:#x}]"),
                            ::core::file!(), #at, #space
                        );
                    }}
                })
            }

            let head = self.out;
            let reader_var = &self.outer_reader_var;
            let ident = &self.field.ident;
            let start_line = start_line(ident.span());
            let at = if start_line == 0 {
                quote!(::core::line!())
            } else {
                start_line.to_token_stream()
            };

            let dbg_pad_before = dbg_space("pad_before", &at, self.field.pad_before.as_ref());
            let dbg_align_before = dbg_space("align_before", &at, self.field.align_before.as_ref());
            let dbg_pad_size_to = dbg_space("pad_size_to", &at, self.field.pad_size_to.as_ref());
            let dbg_pad_after = dbg_space("pad_after", &at, self.field.pad_after.as_ref());
            let dbg_align_after = dbg_space("align_after", &at, self.field.align_after.as_ref());

            self.out = quote! {{
                #dbg_pad_before
                #dbg_align_before
                let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#reader_var)?;
                let #TEMP = #head;
                #DBG_EPRINTLN!(
                    "[{}:{} | offset {:#x}] {} = {:#x?}",
                    ::core::file!(), #at, #SAVED_POSITION, ::core::stringify!(#ident), &#TEMP
                );
                #dbg_pad_size_to
                #dbg_pad_after
                #dbg_align_after
                #TEMP
            }};
        }

        self
    }

    fn append_assertions(mut self) -> Self {
        let assertions = get_assertions(&self.field.assertions);
        let head = self.out;
        self.out = quote! {
            #head
            #(#assertions)*
        };

        self
    }

    fn assign_to_var(mut self) -> Self {
        let ident = &self.field.ident;
        let ty = &self.field.ty;
        let value = self.out;
        self.out = quote! { let mut #ident: #ty = #value; };

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }

    fn map_value(mut self) -> Self {
        let map_func = make_ident(&self.field.ident, "map_func");

        self.out = match &self.field.map {
            Map::None => return self,
            Map::Map(m) => {
                let value = self.out;
                quote_spanned! {m.span()=> #map_func(#value) }
            }
            Map::Try(t) | Map::Repr(t) => {
                // TODO: Position should always just be saved once for a field if used
                let value = self.out;
                let map_err = get_map_err(SAVED_POSITION, t.span());
                let reader_var = &self.outer_reader_var;
                quote_spanned! {t.span()=> {
                    let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#reader_var)?;

                    #map_func(#value)#map_err?
                }}
            }
        };

        self
    }

    fn prefix_map_function(mut self) -> Self {
        let map_func = make_ident(&self.field.ident, "map_func");
        let ty = &self.field.ty;

        let set_map_function = match &self.field.map {
            Map::None => return self,
            Map::Map(map) => {
                quote! {
                    let mut #map_func = (#COERCE_FN::<#ty, _, _>(#map));
                }
            }
            Map::Try(try_map) | Map::Repr(try_map) => {
                let try_map = if matches!(self.field.map, Map::Repr(_)) {
                    quote! {
                        <#try_map as core::convert::TryInto<_>>::try_into
                    }
                } else {
                    try_map.clone()
                };

                // TODO: Position should always just be saved once for a field if used
                quote! {
                    let mut #map_func = (#COERCE_FN::<::core::result::Result<#ty, _>, _, _>(#try_map));
                }
            }
        };

        let rest = self.out;
        self.out = quote! {
            #set_map_function
            #rest
        };

        self
    }

    fn wrap_map_stream(mut self) -> Self {
        if let Some(map_stream) = &self.field.map_stream {
            let rest = self.out;
            let reader_var = &self.reader_var;
            let outer_reader_var = &self.outer_reader_var;
            self.out = quote_spanned_any! { map_stream.span()=> {
                let #reader_var = &mut #MAP_READER_TYPE_HINT::<R, _, _>(#map_stream)(#outer_reader_var);
                #rest
            }};
        }

        self
    }

    fn prefix_read_function(mut self) -> Self {
        let read_function = match &self.field.field_mode {
            FieldMode::Function(parser) => {
                quote_spanned_any! { parser.span()=>
                    let #READ_FUNCTION = #PARSE_FN_TYPE_HINT(#parser);
                }
            }
            FieldMode::Normal => quote! {
                let #READ_FUNCTION = #READ_METHOD;
            },
            _ => return self,
        };

        let rest = self.out;
        self.out = quote! {
            #read_function
            #rest
        };

        self
    }

    fn prefix_args_and_options(mut self) -> Self {
        let args = self.args_var.as_ref().map(|args_var| {
            let map_func = make_ident(&self.field.ident, "map_func");
            let args = get_passed_args(self.field, &self.outer_reader_var);
            let ty = &self.field.ty;

            if let FieldMode::Function(_) = &self.field.field_mode {
                quote_spanned! {ty.span()=>
                    let #args_var = #ARGS_TYPE_HINT::<_, #ty, _, _>(&#READ_FUNCTION, #args);
                }
            } else {
                match &self.field.map {
                    Map::Map(_) | Map::Try(_) | Map::Repr(_) => {
                        quote_spanned! {ty.span()=>
                            let #args_var = #MAP_ARGS_TYPE_HINT(&#map_func, #args);
                        }
                    }
                    Map::None => {
                        quote_spanned! {ty.span()=>
                            let #args_var: <#ty as #BINREAD_TRAIT>::Args<'_> = #args;
                        }
                    }
                }
            }
        });

        let endian = self.field.needs_endian().then(|| {
            let var = &self.endian_var;
            let endian = get_endian(&self.field.endian);
            quote! { let #var = #endian; }
        });

        let tail = self.out;

        self.out = quote! {
            #args
            #endian
            #tail
        };

        self
    }

    fn prefix_magic(mut self) -> Self {
        if let Some(magic) = get_magic(&self.field.magic, &self.outer_reader_var, &self.endian_var)
        {
            let tail = self.out;
            self.out = quote! {
                #magic
                #tail
            };
        }

        self
    }

    fn read_value(mut self) -> Self {
        self.out = match &self.field.field_mode {
            FieldMode::Default => quote! { <_>::default() },
            FieldMode::Calc(calc) => quote! { #calc },
            FieldMode::TryCalc(calc) => get_try_calc(POS, &self.field.ty, calc),
            read_mode @ (FieldMode::Normal | FieldMode::Function(_)) => {
                let args_arg = self.args_var.as_ref().map_or_else(
                    || quote_spanned! {self.field.ty.span()=> <_ as #REQUIRED_ARG_TRAIT>::args() },
                    ToTokens::to_token_stream,
                );
                let reader_var = &self.reader_var;
                let endian_var = &self.endian_var;

                if let FieldMode::Function(f) = read_mode {
                    let ty = &self.field.ty;
                    // Mapping the value with an explicit type ensures the
                    // incompatible type is warned here as a mismatched type
                    // instead of later as a try-conversion error
                    let map = self.field.map.is_none().then(|| {
                        quote_spanned! { f.span()=>
                            .map(|v| -> #ty { v })
                        }
                    });

                    // Adding a closure suppresses mentions of the generated
                    // READ_FUNCTION variable in errors
                    quote_spanned_any! { f.span()=>
                        (|| #READ_FUNCTION)()(#reader_var, #endian_var, #args_arg)
                        #map
                    }
                } else {
                    quote! {
                        #READ_FUNCTION(#reader_var, #endian_var, #args_arg)
                    }
                }
            }
        };

        self
    }

    fn try_conversion(mut self, name: Option<&Ident>, variant_name: Option<&str>) -> Self {
        if !self.field.generated_value() {
            let result = self.out;
            self.out = if self.field.do_try.is_some() {
                quote! { #result.unwrap_or_default() }
            } else {
                let span = match &self.field.field_mode {
                    FieldMode::Function(f) => f.span(),
                    _ => result.span(),
                };

                let map_err = get_err_context(self.field, name, variant_name);
                quote_spanned! {span=> #result #map_err ? }
            };
        }

        self
    }

    fn wrap_condition(mut self) -> Self {
        if let Some(cond) = &self.field.if_cond {
            let condition = &cond.condition;
            let consequent = self.out;
            let alternate = cond
                .alternate
                .as_ref()
                .map_or_else(|| Cow::Owned(quote! { <_>::default() }), Cow::Borrowed);
            self.out = quote! {
                if #condition {
                    #consequent
                } else {
                    #alternate
                }
            };
        }

        self
    }

    fn wrap_restore_position(mut self) -> Self {
        if self.field.restore_position.is_some() {
            self.out = wrap_save_restore(&self.outer_reader_var, self.out);
        }

        self
    }

    fn wrap_seek(mut self) -> Self {
        let seek_before = generate_seek_before(&self.outer_reader_var, self.field);
        let seek_after = generate_seek_after(&self.outer_reader_var, self.field);
        if !seek_before.is_empty() || !seek_after.is_empty() {
            let value = self.out;
            self.out = quote! {{
                #seek_before
                let #TEMP = #value;
                #seek_after
                #TEMP
            }};
        }

        self
    }
}

fn get_err_context(
    field: &StructField,
    name: Option<&Ident>,
    variant_name: Option<&str>,
) -> TokenStream {
    let backtrace = if let Some(ErrContext::Context(expr)) = &field.err_context {
        quote_spanned! {field.ident.span()=>
            #BACKTRACE_FRAME::Custom(Box::new(#expr) as _)
        }
    } else {
        #[cfg(feature = "verbose-backtrace")]
        let code = {
            let code = BacktraceFrame::from_field(field).to_string();
            if code.is_empty() {
                quote! { None }
            } else {
                quote! { Some(#code) }
            }
        };
        #[cfg(not(feature = "verbose-backtrace"))]
        let code = quote!(None);

        let message = if let Some(ErrContext::Format(fmt, exprs)) = &field.err_context {
            if exprs.is_empty() {
                quote! { (#fmt) }
            } else {
                quote! {
                    {
                        extern crate alloc;
                        alloc::format!(#fmt, #(#exprs),*)
                    }
                }
            }
        } else {
            format!(
                "While parsing field '{}' in {}",
                field.ident,
                name.map_or_else(|| variant_name.unwrap().into(), ToString::to_string)
            )
            .into_token_stream()
        };

        quote_spanned! {field.ident.span()=>
            #BACKTRACE_FRAME::Full {
                message: #message.into(),
                line: ::core::line!(),
                file: ::core::file!(),
                code: #code,
            }
        }
    };

    quote! {
        .map_err(|err| #WITH_CONTEXT(err, #backtrace))
    }
}

fn get_prelude(input: &Input, name: Option<&Ident>) -> TokenStream {
    PreludeGenerator::new(input)
        .add_imports(name)
        .add_endian()
        .add_magic_pre_assertion()
        .add_map_stream()
        .finish()
}

fn generate_seek_after(reader_var: &TokenStream, field: &StructField) -> TokenStream {
    let pad_size_to = field.pad_size_to.as_ref().map(|pad| {
        quote! {{
            let pad = (#pad) as i64;
            let size = (#SEEK_TRAIT::stream_position(#reader_var)? - #POS) as i64;
            if size < pad {
                #SEEK_TRAIT::seek(#reader_var, #SEEK_FROM::Current(pad - size))?;
            }
        }}
    });
    let pad_after = field
        .pad_after
        .as_ref()
        .map(|value| map_pad(reader_var, value));
    let align_after = field
        .align_after
        .as_ref()
        .map(|value| map_align(reader_var, value));

    quote! {
        #pad_size_to
        #pad_after
        #align_after
    }
}

fn generate_seek_before(reader_var: &TokenStream, field: &StructField) -> TokenStream {
    let seek_before = field.seek_before.as_ref().map(|seek| {
        quote! {
            #SEEK_TRAIT::seek(#reader_var, #seek)?;
        }
    });
    let pad_before = field
        .pad_before
        .as_ref()
        .map(|value| map_pad(reader_var, value));
    let align_before = field
        .align_before
        .as_ref()
        .map(|value| map_align(reader_var, value));
    let pad_size_to_before = field.pad_size_to.as_ref().map(|_| {
        quote! {
            let #POS = #SEEK_TRAIT::stream_position(#reader_var)?;
        }
    });

    quote! {
        #seek_before
        #pad_before
        #align_before
        #pad_size_to_before
    }
}

fn get_return_type(variant_ident: Option<&Ident>) -> TokenStream {
    variant_ident.map_or_else(|| quote! { Self }, |ident| quote! { Self::#ident })
}

fn make_field_vars(
    input: &Input,
    field: &StructField,
) -> (TokenStream, TokenStream, Option<Ident>) {
    let reader_var = if field.map_stream.is_some() {
        make_ident(&field.ident, "reader").into_token_stream()
    } else {
        input.stream_ident_or(READER)
    };

    let endian_var = if field.needs_endian() {
        make_ident(&field.ident, "endian").into_token_stream()
    } else {
        OPT.to_token_stream()
    };

    let args_var = if field.needs_args() {
        Some(make_ident(&field.ident, "args"))
    } else {
        None
    };

    (reader_var, endian_var, args_var)
}

fn map_align(reader_var: &TokenStream, align: &TokenStream) -> TokenStream {
    quote! {{
        let align = (#align) as i64;
        let pos = #SEEK_TRAIT::stream_position(#reader_var)? as i64;
        #SEEK_TRAIT::seek(#reader_var, #SEEK_FROM::Current((align - (pos % align)) % align))?;
    }}
}

fn map_pad(reader_var: &TokenStream, pad: &TokenStream) -> TokenStream {
    quote! {
        #SEEK_TRAIT::seek(#reader_var, #SEEK_FROM::Current((#pad) as i64))?;
    }
}

fn wrap_save_restore(reader_var: &TokenStream, value: TokenStream) -> TokenStream {
    if value.is_empty() {
        value
    } else {
        quote! {
            let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#reader_var)?;
            #value
            #SEEK_TRAIT::seek(#reader_var, #SEEK_FROM::Start(#SAVED_POSITION))?;
        }
    }
}
