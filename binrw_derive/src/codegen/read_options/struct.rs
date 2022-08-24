use super::{get_assertions, get_magic, PreludeGenerator, ReadOptionsGenerator};
#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::{ErrContext, FieldMode, Map, PassedArgs};
use crate::parser::{Input, Struct, StructField};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{spanned::Spanned, Ident};

#[cfg(all(nightly, not(coverage)))]
use crate::backtrace::BacktraceFrame;

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
            .map(|field| generate_field(field, name, variant_name));
        let after_parse = {
            let after_parse = self.st.fields.iter().map(generate_after_parse);
            wrap_save_restore(quote!(#(#after_parse)*))
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

fn generate_after_parse(field: &StructField) -> Option<TokenStream> {
    if field.deref_now.is_none() {
        get_after_parse_handler(field).map(|after_parse_fn| {
            let (args_var, options_var) = make_field_vars(field);
            AfterParseCallGenerator::new(field)
                .get_value_from_ident()
                .call_after_parse(after_parse_fn, &options_var, &args_var)
                .prefix_offset_options(&options_var)
                .finish()
        })
    } else {
        None
    }
}

fn generate_field(
    field: &StructField,
    name: Option<&Ident>,
    variant_name: Option<&str>,
) -> TokenStream {
    // temp + ignore == just don't bother
    if field.is_temp(false) && matches!(field.read_mode, FieldMode::Default) {
        return TokenStream::new();
    }

    FieldGenerator::new(field)
        .read_value()
        .try_conversion(name, variant_name)
        .map_value()
        .deref_now()
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
        options_var: &Option<Ident>,
        args_var: &Option<Ident>,
    ) -> Self {
        let value = self.out;
        let options_var = options_var.as_ref().expect(
            "called `AfterParseCallGenerator::call_after_parse` but no `options_var` was generated",
        );
        let args_arg = get_args_argument(args_var);
        self.out = quote! {
            #after_parse_fn(#value, #READER, #options_var, #args_arg)?;
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

    fn prefix_offset_options(mut self, options_var: &Option<Ident>) -> Self {
        if let (Some(options_var), Some(offset)) = (options_var, &self.field.offset_after) {
            let tail = self.out;
            let offset = offset.as_ref();
            self.out = quote! {
                let #options_var = &{
                    let mut #TEMP = *#options_var;
                    let #TEMP = #TEMP.with_offset(#offset);
                    #TEMP
                };
                #tail
            };
        }

        self
    }
}

struct FieldGenerator<'field> {
    field: &'field StructField,
    out: TokenStream,
    args_var: Option<Ident>,
    options_var: Option<Ident>,
}

impl<'field> FieldGenerator<'field> {
    fn new(field: &'field StructField) -> Self {
        let (args_var, options_var) = make_field_vars(field);

        Self {
            field,
            out: TokenStream::new(),
            args_var,
            options_var,
        }
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
                .call_after_parse(after_parse, &self.options_var, &self.args_var)
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
            Map::Try(_) | Map::Repr(_) => {
                // TODO: Position should always just be saved once for a field if used
                let value = self.out;
                let map_err = super::get_map_err(SAVED_POSITION);
                quote! {{
                    let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#READER)?;

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
                    let #map_func = (#COERCE_FN::<#ty, _, _>(#map));
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
                    let #map_func = (#COERCE_FN::<::core::result::Result<#ty, _>, _, _>(#try_map));
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

    fn prefix_read_function(mut self) -> Self {
        let read_function = match &self.field.read_mode {
            FieldMode::Function(parser) => Some(parser.clone()),
            FieldMode::Normal => Some(READ_METHOD.to_token_stream()),
            _ => None,
        };

        let rest = self.out;
        self.out = quote! {
            let #READ_FUNCTION = (#read_function);
            #rest
        };

        self
    }

    fn prefix_args_and_options(mut self) -> Self {
        let args = self.args_var.as_ref().map(|args_var| {
            let map_func = make_ident(&self.field.ident, "map_func");
            let args = get_passed_args(self.field);
            let ty = &self.field.ty;

            if let FieldMode::Function(_) = &self.field.read_mode {
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
                            let #args_var: <#ty as #BINREAD_TRAIT>::Args = #args;
                        }
                    }
                }
            }
        });

        let options = self.options_var.as_ref().map(|options_var| {
            ReadOptionsGenerator::new(options_var)
                .endian(&self.field.endian)
                .offset(&self.field.offset)
                .finish()
        });

        let tail = self.out;

        self.out = quote! {
            #args
            #options
            #tail
        };

        self
    }

    fn prefix_magic(mut self) -> Self {
        if let Some(options_var) = &self.options_var {
            if let Some(magic) = get_magic(&self.field.magic, options_var) {
                let tail = self.out;
                self.out = quote! {
                    #magic
                    #tail
                };
            }
        }

        self
    }

    fn read_value(mut self) -> Self {
        self.out = match &self.field.read_mode {
            FieldMode::Default => quote! { <_>::default() },
            FieldMode::Calc(calc) => quote! { #calc },
            FieldMode::Normal | FieldMode::Function(_) => {
                let args_arg = get_args_argument(&self.args_var);
                let options_var = &self.options_var;

                quote! {
                    #READ_FUNCTION(#READER, #options_var, #args_arg)
                }
            }
        };

        self
    }

    fn map_err_context(&self, name: Option<&Ident>, variant_name: Option<&str>) -> TokenStream {
        #[cfg(all(nightly, not(coverage)))]
        let code = {
            let code = BacktraceFrame::from_field(self.field).to_string();
            quote!(Some(#code))
        };

        #[cfg(any(not(nightly), coverage))]
        let code = quote!(None);

        match self.field.err_context.as_ref() {
            Some(ErrContext::Format(message, exprs)) if exprs.is_empty() => {
                quote_spanned!( self.field.ident.span() =>
                    .map_err(|err| #WITH_CONTEXT(err, #BACKTRACE_FRAME::Full {
                        message: #message,
                        line: ::core::line!(),
                        file: ::core::file!(),
                        code: #code,
                    }))
                )
            }
            Some(ErrContext::Format(format, exprs)) => {
                quote_spanned!( self.field.ident.span() =>
                    .map_err(|err| #WITH_CONTEXT(err, #BACKTRACE_FRAME::OwnedFull {
                        message: ::binrw::alloc::format!(#format, #(#exprs),*),
                        line: ::core::line!(),
                        file: ::core::file!(),
                        code: #code,
                    }))
                )
            }
            Some(ErrContext::Context(expr)) => {
                quote!(
                    .map_err(|err| #WITH_CONTEXT(err, #BACKTRACE_FRAME::Custom(Box::new(#expr) as _)))
                )
            }
            None => {
                let message = format!(
                    "While parsing field '{}' in {}",
                    self.field.ident,
                    name.map_or_else(
                        || variant_name
                            .unwrap_or("[please report this error]")
                            .to_string(),
                        ToString::to_string
                    )
                );

                quote_spanned!( self.field.ident.span() =>
                    .map_err(|err| #WITH_CONTEXT(err, #BACKTRACE_FRAME::Full {
                        message: #message,
                        line: ::core::line!(),
                        file: ::core::file!(),
                        code: #code,
                    }))
                )
            }
        }
    }

    fn try_conversion(mut self, name: Option<&Ident>, variant_name: Option<&str>) -> Self {
        if !self.field.generated_value() {
            let result = &self.out;
            self.out = if self.field.do_try.is_some() {
                quote! { #result.unwrap_or(<_>::default()) }
            } else {
                let map_err = self.map_err_context(name, variant_name);
                quote! { #result #map_err ? }
            };
        }

        self
    }

    fn wrap_condition(mut self) -> Self {
        if let Some(cond) = &self.field.if_cond {
            let condition = &cond.condition;
            let consequent = self.out;
            let alternate = &cond.alternate;
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
            self.out = wrap_save_restore(self.out);
        }

        self
    }

    fn wrap_seek(mut self) -> Self {
        let seek_before = generate_seek_before(self.field);
        let seek_after = generate_seek_after(self.field);
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

fn get_args_argument(args_var: &Option<Ident>) -> TokenStream {
    args_var.as_ref().map_or_else(
        || quote! { <_>::default() },
        |args_var| quote! { #args_var.clone() },
    )
}

fn get_passed_args(field: &StructField) -> Option<TokenStream> {
    let args = &field.args;
    match args {
        PassedArgs::Named(fields) => Some(if let Some(count) = &field.count {
            // quote-spanning changes the resolution behaviour such that clippy
            // thinks `(#count) as usize` is part of the source code instead of
            // generated code, so instead only set the span on the fields-part
            // to try to get the error reporting benefits without the incorrect
            // lints
            let fields = quote_spanned! {fields.span()=> #(, #fields)* };

            quote! {
                #ARGS_MACRO! { count: ((#count) as usize) #fields }
            }
        } else {
            quote_spanned! {fields.span()=>
                #ARGS_MACRO! { #(#fields),* }
            }
        }),
        PassedArgs::List(list) => Some(quote_spanned! {list.span()=> (#(#list,)*) }),
        PassedArgs::Tuple(tuple) => {
            let tuple = tuple.as_ref();
            Some(quote_spanned! {tuple.span()=> #tuple })
        }
        PassedArgs::None => field
            .count
            .as_ref()
            .map(|count| quote! { #ARGS_MACRO! { count: ((#count) as usize) }}),
    }
}

fn get_prelude(input: &Input, name: Option<&Ident>) -> TokenStream {
    PreludeGenerator::new(input)
        .add_imports(name)
        .add_options()
        .add_magic_pre_assertion()
        .finish()
}

fn generate_seek_after(field: &StructField) -> TokenStream {
    let pad_size_to = field.pad_size_to.as_ref().map(|pad| {
        quote! {{
            let pad = (#pad) as i64;
            let size = (#SEEK_TRAIT::stream_position(#READER)? - #POS) as i64;
            if size < pad {
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(pad - size))?;
            }
        }}
    });
    let pad_after = field.pad_after.as_ref().map(map_pad);
    let align_after = field.align_after.as_ref().map(map_align);

    quote! {
        #pad_size_to
        #pad_after
        #align_after
    }
}

fn generate_seek_before(field: &StructField) -> TokenStream {
    let seek_before = field.seek_before.as_ref().map(|seek| {
        quote! {
            #SEEK_TRAIT::seek(#READER, #seek)?;
        }
    });
    let pad_before = field.pad_before.as_ref().map(map_pad);
    let align_before = field.align_before.as_ref().map(map_align);
    let pad_size_to_before = field.pad_size_to.as_ref().map(|_| {
        quote! {
            let #POS = #SEEK_TRAIT::stream_position(#READER)?;
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

fn make_field_vars(field: &StructField) -> (Option<Ident>, Option<Ident>) {
    let args_var = if field.args.is_some() || field.count.is_some() {
        Some(make_ident(&field.ident, "args"))
    } else {
        None
    };

    let options_var = if field.needs_options() {
        Some(make_ident(&field.ident, "options"))
    } else {
        None
    };

    (args_var, options_var)
}

fn map_align(align: &TokenStream) -> TokenStream {
    quote! {{
        let align = (#align) as i64;
        let pos = #SEEK_TRAIT::stream_position(#READER)? as i64;
        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current((align - (pos % align)) % align))?;
    }}
}

fn map_pad(pad: &TokenStream) -> TokenStream {
    quote! {
        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current((#pad) as i64))?;
    }
}

fn wrap_save_restore(value: TokenStream) -> TokenStream {
    if value.is_empty() {
        value
    } else {
        quote! {
            let #SAVED_POSITION = #SEEK_TRAIT::stream_position(#READER)?;
            #value
            #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))?;
        }
    }
}
