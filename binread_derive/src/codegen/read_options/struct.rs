#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::{Input, Map, PassedArgs, ReadMode, Struct, StructField};
use proc_macro2::TokenStream;
use quote::quote;
use super::{PreludeGenerator, ReadOptionsGenerator, debug_template, get_assertions};
use syn::Ident;

pub(super) fn generate_unit_struct(input: &Input, variant_ident: Option<&Ident>) -> TokenStream {
    let prelude = get_prelude(input);
    let return_type = get_return_type(variant_ident);
    quote! {
        #prelude
        Ok(#return_type)
    }
}

pub(super) fn generate_struct(ident: &Ident, input: &Input, st: &Struct) -> TokenStream {
    StructGenerator::new(input, st)
        .read_fields()
        .wrap_debug(ident)
        .add_assertions(core::iter::empty())
        .return_value(None)
        .finish()
}

pub(super) struct StructGenerator<'input> {
    input: &'input Input,
    st: &'input Struct,
    out: TokenStream,
}

impl <'input> StructGenerator<'input> {
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

    pub(super) fn add_assertions(mut self, extra_assertions: impl Iterator<Item = TokenStream>) -> Self {
        let assertions = get_assertions(&self.st.assertions).chain(extra_assertions);
        let head = self.out;
        self.out = quote! {
            #head
            #(#assertions)*
        };

        self
    }

    pub(super) fn read_fields(mut self) -> Self {
        let prelude = get_prelude(self.input);
        let read_fields = self.st.fields.iter().map(|field| generate_field(field));
        let after_parse = {
            let after_parse = self.st.fields.iter().map(|field| generate_after_parse(field));
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

    pub(super) fn wrap_debug(mut self, ident: &Ident) -> Self {
        if cfg!(feature = "debug_template") {
            let debug_tpl_start = debug_template::start(&ident);
            let debug_tpl_end = debug_template::end();
            let body = self.out;
            self.out = quote! {
                #debug_tpl_start
                #body
                #debug_tpl_end
            };
        }

        self
    }
}

fn generate_after_parse(field: &StructField) -> Option<TokenStream> {
    if field.deref_now || field.postprocess_now {
        None
    } else {
        get_after_parse_handler(&field).map(|after_parse_fn| {
            let args_var = make_ident(&field.ident, "args");
            let options_var = make_ident(&field.ident, "options");
            AfterParseGenerator::new(field)
                .get_value_from_ident()
                .call_after_parse(after_parse_fn, &options_var, &args_var)
                .wrap_condition()
                .prefix_offset_options(&options_var)
                .finish()
        })
    }
}

fn generate_field(field: &StructField) -> TokenStream {
    let args_var = make_ident(&field.ident, "args");
    let options_var = make_ident(&field.ident, "options");
    FieldGenerator::new(&field)
        .read_value(&options_var, &args_var)
        .try_conversion()
        .map_value()
        .deref_now(&options_var, &args_var)
        .wrap_seek()
        .wrap_condition()
        .assign_to_var()
        .append_assertions()
        .wrap_restore_position()
        .prefix_args_and_options(&options_var, &args_var)
        .finish()
}

struct AfterParseGenerator<'field> {
    field: &'field StructField,
    out: TokenStream,
}

impl <'field> AfterParseGenerator<'field> {
    fn new(field: &'field StructField) -> Self {
        Self {
            field,
            out: TokenStream::new(),
        }
    }

    fn call_after_parse(mut self, after_parse_fn: IdentStr, options_var: &Ident, args_var: &Ident) -> Self {
        let handle_error = debug_template::handle_error();
        let value = self.out;
        self.out = quote! {
            #after_parse_fn(#value, #READER, #options_var, #args_var.clone())#handle_error?;
        };

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }

    fn get_value_from_ident(mut self) -> Self {
        let ident = &self.field.ident;
        self.out = if self.field.if_cond.is_some() {
            quote! { #ident }
        } else {
            quote! { &mut #ident }
        };

        self
    }

    fn get_value_from_temp(mut self) -> Self {
        self.out = quote! { &mut #TEMP };

        self
    }

    fn prefix_offset_options(mut self, options_var: &Ident) -> Self {
        if let Some(offset) = &self.field.offset_after {
            let tail = self.out;
            self.out = quote! {
                let #options_var = &{
                    let mut #TEMP = *#options_var;
                    #TEMP.offset = #offset;
                    #TEMP
                };
                #tail
            };
        }

        self
    }

    fn wrap_condition(mut self) -> Self {
        if self.field.if_cond.is_some() {
            let ident = &self.field.ident;
            let body = self.out;
            self.out = quote! {
                if let Some(#ident) = #ident.as_mut() {
                    #body
                }
            };
        }

        self
    }
}

struct FieldGenerator<'field> {
    field: &'field StructField,
    out: TokenStream,
    emit_options_vars: bool,
}

impl <'field> FieldGenerator<'field> {
    fn new(field: &'field StructField) -> Self {
        Self {
            field,
            out: TokenStream::new(),
            emit_options_vars: get_after_parse_handler(field).is_some(),
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

    fn deref_now(mut self, options_var: &Ident, args_var: &Ident) -> Self {
        if !self.field.deref_now && !self.field.postprocess_now {
            return self;
        }

        if let Some(after_parse) = get_after_parse_handler(&self.field) {
            let after_parse = AfterParseGenerator::new(self.field)
                .get_value_from_temp()
                .call_after_parse(after_parse, options_var, args_var)
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
        // TODO: Coerce function should just be emitted once, or put into the
        // binread library instead

        // This validates the map function return value by trying to coerce it into
        // a function with the expected return type. If this is not done, the
        // compiler will emit the diagnostic on the `#[derive(BinRead)]` attribute
        // instead of the return statement of the map function. The simpler approach
        // of assigning the map function to a variable with a function pointer type
        // does not work for capturing closures since they are not compatible with
        // that type.
        let coerce_fn = quote! {
            fn __binread_coerce<R, T, F>(f: F) -> F where F: Fn(T) -> R { f }
        };

        let ty = &self.field.ty;

        self.out = match &self.field.map {
            Map::None => return self,
            Map::Map(map) => {
                let value = self.out;
                quote! {{
                    #coerce_fn
                    (__binread_coerce::<#ty, _, _>(#map))(#value)
                }}
            },
            Map::Try(try_map) => {
                // TODO: Position should always just be saved once for a field if used
                let value = self.out;
                quote! {{
                    let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))?;

                    #coerce_fn
                    (__binread_coerce::<::core::result::Result<#ty, _>, _, _>(#try_map))(#value).map_err(|e| {
                        #BIN_ERROR::Custom {
                            pos: #SAVED_POSITION as _,
                            err: Box::new(e) as _,
                        }
                    })?
                }}
            },
        };

        self
    }

    fn prefix_args_and_options(mut self, options_var: &Ident, args_var: &Ident) -> Self {
        if self.emit_options_vars {
            let args = get_passed_args(&self.field.args);
            let options = ReadOptionsGenerator::new(options_var)
                .endian(&self.field.endian)
                .offset(&self.field.offset)
                .variable_name(&self.field.ident)
                .count(&self.field.count)
                .finish();
            let tail = self.out;
            self.out = quote! {
                let #args_var = #args;
                #options
                #tail
            };
        }

        self
    }

    fn read_value(mut self, options_var: &Ident, args_var: &Ident) -> Self {
        self.out = match &self.field.read_mode {
            ReadMode::Default => quote! { <_>::default() },
            ReadMode::Calc(calc) => quote! { #calc },
            ReadMode::Normal | ReadMode::ParseWith(_) => {
                let read_method = if let ReadMode::ParseWith(parser) = &self.field.read_mode {
                    parser.clone()
                } else {
                    quote! { #READ_METHOD }
                };

                self.emit_options_vars = true;

                quote! {
                    #read_method(#READER, #options_var, #args_var.clone())
                }
            }
        };

        self
    }

    fn try_conversion(mut self) -> Self {
        if self.field.generated_value() {
            if self.field.do_try {
                let value = self.out;
                self.out = quote! { Some(#value) };
            }
        } else {
            let result = self.out;
            self.out = if self.field.do_try {
                quote! { #result.ok() }
            } else {
                let handle_error = debug_template::handle_error();
                quote! { #result#handle_error? }
            };
        }

        self
    }

    fn wrap_condition(mut self) -> Self {
        if let Some(cond) = &self.field.if_cond {
            let value = self.out;
            self.out = quote! {
                if #cond {
                    Some(#value)
                } else {
                    None
                }
            };
        }

        self
    }

    fn wrap_restore_position(mut self) -> Self {
        if self.field.restore_position {
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

fn get_passed_args(args: &PassedArgs) -> TokenStream {
    match args {
        PassedArgs::List(list) => quote! { (#(#list,)*) },
        PassedArgs::Tuple(tuple) => tuple.clone(),
        PassedArgs::None => quote! { () },
    }
}

fn get_prelude(input: &Input) -> TokenStream {
    PreludeGenerator::new(input)
        .add_imports()
        .add_options()
        .add_magic_pre_assertion()
        .finish()
}

fn generate_seek_after(field: &StructField) -> TokenStream {
    let handle_error = debug_template::handle_error();
    let pad_size_to = field.pad_size_to.as_ref().map(|pad| quote! {{
        let pad = (#pad) as i64;
        let size = (#SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error? - #POS) as i64;
        if size < pad {
            #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(pad - size))#handle_error?;
        }
    }});
    let pad_after = field.pad_after.as_ref().map(map_pad);
    let align_after = field.align_after.as_ref().map(map_align);

    quote! {
        #pad_size_to
        #pad_after
        #align_after
    }
}

fn generate_seek_before(field: &StructField) -> TokenStream {
    let handle_error = debug_template::handle_error();
    let seek_before = field.seek_before.as_ref().map(|seek| quote! {
        #SEEK_TRAIT::seek(#READER, #seek)#handle_error?;
    });
    let pad_before = field.pad_before.as_ref().map(map_pad);
    let align_before = field.align_before.as_ref().map(map_align);
    let pad_size_to_before = field.pad_size_to.as_ref().map(|_| quote! {
        let #POS = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error?;
    });

    quote! {
        #seek_before
        #pad_before
        #align_before
        #pad_size_to_before
    }
}

fn get_after_parse_handler(field: &StructField) -> Option<IdentStr> {
    if !field.can_call_after_parse() {
        None
    } else if field.do_try {
        Some(AFTER_PARSE_TRY)
    } else {
        Some(AFTER_PARSE)
    }
}

fn get_return_type(variant_ident: Option<&Ident>) -> TokenStream {
    variant_ident.map_or_else(
        || quote! { Self },
        |ident| quote! { Self::#ident }
    )
}

fn map_align(align: &TokenStream) -> TokenStream {
    let handle_error = debug_template::handle_error();
    quote! {{
        let align = (#align) as i64;
        let pos = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error? as i64;
        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current((align - (pos % align)) % align))#handle_error?;
    }}
}

fn map_pad(pad: &TokenStream) -> TokenStream {
    let handle_error = debug_template::handle_error();
    quote! {
        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(#pad))#handle_error?;
    }
}

fn wrap_save_restore(value: TokenStream) -> TokenStream {
    if value.is_empty() {
        value
    } else {
        let handle_error = debug_template::handle_error();
        quote! {
            let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error?;
            #value
            #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))#handle_error?;
        }
    }
}
