use super::{get_magic, PreludeGenerator};
#[cfg(feature = "verbose-backtrace")]
use crate::binrw::backtrace::BacktraceFrame;
use crate::{
    binrw::{
        codegen::{
            get_assertions, get_endian, get_map_err, get_passed_args, get_try_calc,
            sanitization::{
                make_ident, AFTER_PARSE, ARGS_MACRO, ARGS_TYPE_HINT, BACKTRACE_FRAME,
                BINREAD_TRAIT, COERCE_FN, DBG_EPRINTLN, MAP_ARGS_TYPE_HINT, MAP_READER_TYPE_HINT,
                OPT, PARSE_FN_TYPE_HINT, POS, READER, READ_FUNCTION, READ_METHOD,
                REQUIRED_ARG_TRAIT, SAVED_POSITION, SEEK_FROM, SEEK_TRAIT, TEMP, WITH_CONTEXT,
            },
        },
        parser::{ErrContext, FieldMode, Input, Map, Struct, StructField},
    },
    util::{quote_spanned_any, IdentStr},
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
        .add_assertions(core::iter::empty())
        .return_value(None)
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

    pub(super) fn add_assertions(
        mut self,
        extra_assertions: impl Iterator<Item = TokenStream>,
    ) -> Self {
        let assertions = get_assertions(&self.st.assertions).chain(extra_assertions);
        let head = self.out;
        self.out = quote! {
            #head
            #(#assertions)*
        };

        self
    }

    pub(super) fn read_fields(mut self, name: Option<&Ident>, variant_name: Option<&str>) -> Self {
        let prelude = get_prelude(self.input, name);
        let read_fields = self
            .st
            .fields
            .iter()
            .map(|field| generate_field(self.input, field, name, variant_name));
        let after_parse = {
            let after_parse = self
                .st
                .fields
                .iter()
                .map(|field| generate_after_parse(self.input, field));
            wrap_save_restore(
                &self.input.stream_ident_or(READER),
                quote!(#(#after_parse)*),
            )
        };
        self.out = quote! {
            #prelude
            #(#read_fields)*
            #after_parse
        };

        self
    }

    pub(super) fn return_value(mut self, variant_ident: Option<&Ident>) -> Self {
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
            Ok(#return_value)
        };

        self
    }
}

fn generate_after_parse(input: &Input, field: &StructField) -> Option<TokenStream> {
    if field.deref_now.is_none() {
        get_after_parse_handler(field).map(|after_parse_fn| {
            let (reader_var, endian_var, args_var) = make_field_vars(input, field);
            AfterParseCallGenerator::new(field)
                .get_value_from_ident()
                .call_after_parse(after_parse_fn, &reader_var, &endian_var, &args_var)
                .finish()
        })
    } else {
        None
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
        .try_conversion(name, variant_name)
        .map_value()
        .deref_now()
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
        .prefix_map_stream()
        .finish()
}

struct AfterParseCallGenerator<'field> {
    field: &'field StructField,
    out: TokenStream,
}

impl<'field> AfterParseCallGenerator<'field> {
    fn new(field: &'field StructField) -> Self {
        Self {
            field,
            out: TokenStream::new(),
        }
    }

    fn call_after_parse(
        mut self,
        after_parse_fn: IdentStr,
        reader_var: &TokenStream,
        endian_var: &TokenStream,
        args_var: &Option<Ident>,
    ) -> Self {
        let value = self.out;
        let args_arg = if let Some(offset) = &self.field.offset_after {
            let offset = offset.as_ref();
            if let Some(args_var) = args_var {
                quote_spanned_any! { offset.span()=> {
                    let mut #TEMP = #args_var;
                    #TEMP.offset = #offset;
                    #TEMP
                }}
            } else {
                quote_spanned_any! { offset.span()=>
                    #ARGS_MACRO! { offset: #offset }
                }
            }
        } else {
            get_args_argument(self.field, args_var, false)
        };

        self.out = quote! {
            #after_parse_fn(#value, #reader_var, #endian_var, #args_arg)?;
        };

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }

    fn get_value_from_ident(mut self) -> Self {
        let ident = &self.field.ident;
        self.out = quote! { &mut #ident };

        self
    }

    fn get_value_from_temp(mut self) -> Self {
        self.out = quote! { &mut #TEMP };

        self
    }
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
            let head = self.out;
            let reader_var = &self.reader_var;
            let ident = &self.field.ident;
            let start_line = start_line(ident.span());
            let at = if start_line == 0 {
                quote!(::core::line!())
            } else {
                start_line.to_token_stream()
            };

            self.out = quote! {{
                let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#reader_var)?;
                let #TEMP = #head;
                #DBG_EPRINTLN!(
                    "[{}:{} | offset {:#x}] {} = {:#x?}",
                    ::core::file!(), #at, #SAVED_POSITION, ::core::stringify!(#ident), &#TEMP
                );
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

    fn deref_now(mut self) -> Self {
        if self.field.should_use_after_parse() {
            return self;
        }

        if let Some(after_parse) = get_after_parse_handler(self.field) {
            let after_parse = AfterParseCallGenerator::new(self.field)
                .get_value_from_temp()
                .call_after_parse(
                    after_parse,
                    &self.reader_var,
                    &self.endian_var,
                    &self.args_var,
                )
                .finish();

            let value = self.out;
            self.out = quote! {{
                let mut #TEMP = #value;
                #after_parse
                #TEMP
            }};
        }

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }

    fn map_value(mut self) -> Self {
        let map_func = make_ident(&self.field.ident, "map_func");

        self.out = match &self.field.map {
            Map::None => return self,
            Map::Map(_) => {
                let value = self.out;
                quote! { #map_func(#value) }
            }
            Map::Try(t) | Map::Repr(t) => {
                // TODO: Position should always just be saved once for a field if used
                let value = self.out;
                let map_err = get_map_err(SAVED_POSITION, t.span());
                let reader_var = &self.reader_var;
                quote! {{
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

    fn prefix_map_stream(mut self) -> Self {
        if let Some(map_stream) = &self.field.map_stream {
            let rest = self.out;
            let reader_var = &self.reader_var;
            let outer_reader_var = &self.outer_reader_var;
            self.out = quote_spanned_any! { map_stream.span()=>
                let #reader_var = &mut #MAP_READER_TYPE_HINT::<R, _, _>(#map_stream)(#outer_reader_var);
                #rest
            };
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
            let args = get_passed_args(self.field, READER);
            let ty = &self.field.ty;

            if let FieldMode::Function(_) = &self.field.field_mode {
                quote_spanned! {ty.span()=>
                    let #args_var = #ARGS_TYPE_HINT::<R, #ty, _, _>(#READ_FUNCTION, #args);
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
        if let Some(magic) = get_magic(&self.field.magic, &self.reader_var, &self.endian_var) {
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
                let args_arg = get_args_argument(
                    self.field,
                    &self.args_var,
                    matches!(read_mode, FieldMode::Normal),
                );
                let reader_var = &self.reader_var;
                let endian_var = &self.endian_var;

                if let FieldMode::Function(f) = read_mode {
                    let ty = &self.field.ty;
                    // Adding a closure suppresses mentions of the generated
                    // READ_FUNCTION variable in errors; mapping the value with
                    // an explicit type ensures the incompatible type is warned
                    // here as a mismatched type instead of later as a
                    // try-conversion error
                    quote_spanned_any! { f.span()=>
                        (|| #READ_FUNCTION)()(#reader_var, #endian_var, #args_arg)
                            .map(|v| -> #ty { v })
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
                let map_err = get_err_context(self.field, name, variant_name);
                quote! { #result #map_err ? }
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
            self.out = wrap_save_restore(&self.reader_var, self.out);
        }

        self
    }

    fn wrap_seek(mut self) -> Self {
        let seek_before = generate_seek_before(&self.reader_var, self.field);
        let seek_after = generate_seek_after(&self.reader_var, self.field);
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

fn get_args_argument(
    field: &StructField,
    args_var: &Option<Ident>,
    need_clone: bool,
) -> TokenStream {
    args_var.as_ref().map_or_else(
        || quote_spanned! {field.ty.span()=> <_ as #REQUIRED_ARG_TRAIT>::args() },
        |args_var| {
            if need_clone {
                quote! { #args_var.clone() }
            } else {
                quote! { #args_var }
            }
        },
    )
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

fn get_after_parse_handler(field: &StructField) -> Option<IdentStr> {
    field.can_call_after_parse().then_some(AFTER_PARSE)
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
