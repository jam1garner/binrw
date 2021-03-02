#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::{Input, Map, Struct, StructField};
use proc_macro2::TokenStream;
use quote::quote;
use super::{
    debug_template,
    get_assertions,
    get_endian_tokens,
    get_magic_pre_assertion,
    get_passed_args,
    get_prelude,
    get_read_options_override_keys,
    get_read_options_with_endian,
};
use syn::Ident;

pub(super) fn generate_unit_struct(tla: &Input) -> TokenStream {
    // TODO: If this is only using endian, magic, and pre_assert, then it is
    // just like a unit enum field and should be parsed and handled that way.
    let imports = tla.imports().idents();
    let top_level_option = get_read_options_with_endian(&tla.endian());
    let magic_handler = get_magic_pre_assertion(&tla);

    quote! {
        let #imports = #ARGS;
        let #OPT = #top_level_option;
        #magic_handler
        Ok(Self)
    }
}

pub(super) fn generate_struct(ident: &Ident, tla: &Input, ds: &Struct) -> TokenStream {
    let debug_tpl_start = debug_template::start(&ident);
    let read_body = generate_body(tla, &ds.fields);
    let debug_tpl_end = debug_template::end();
    let assertions = get_assertions(&ds.assert);
    let out_names = ds.iter_permanent_idents();
    let return_value = if ds.is_tuple() {
        quote! { Self(#(#out_names),*) }
    } else {
        quote! { Self { #(#out_names),* } }
    };

    quote! {
        #debug_tpl_start
        #read_body
        #debug_tpl_end
        #(#assertions)*
        Ok(#return_value)
    }
}

// TODO: Should not be public
pub(super) fn generate_body(tla: &Input, fields: &[StructField]) -> TokenStream {
    let prelude = get_prelude(tla);
    let read_fields = fields.iter().map(|field| generate_field(field));
    let after_parse = {
        let after_parse = fields.iter().map(|field| generate_after_parse(field));
        wrap_save_restore(quote!(#(#after_parse)*))
    };
    quote! {
        #prelude
        #(#read_fields)*
        #after_parse
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
                .get_value()
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

struct AfterParseGenerator<'field>(&'field StructField, TokenStream);

impl <'field> AfterParseGenerator<'field> {
    fn new(field: &'field StructField) -> Self {
        Self(field, TokenStream::new())
    }

    fn call_after_parse(mut self, after_parse_fn: IdentStr, options_var: &Ident, args_var: &Ident) -> Self {
        let handle_error = debug_template::handle_error();
        let value = &self.1;
        self.1 = quote! {
            #after_parse_fn(#value, #READER, #options_var, #args_var.clone())#handle_error?;
        };

        self
    }

    fn finish(self) -> TokenStream {
        self.1
    }

    fn get_temp_value(mut self) -> Self {
        self.1 = quote! { &mut #TEMP };

        self
    }

    fn get_value(mut self) -> Self {
        let ident = &self.0.ident;
        self.1 = if self.0.if_cond.is_some() {
            quote! { #ident }
        } else {
            quote! { &mut #ident }
        };

        self
    }

    fn prefix_offset_options(mut self, options_var: &Ident) -> Self {
        if let Some(offset) = &self.0.offset_after {
            let value = &self.1;
            self.1 = quote! {
                let #options_var = &{
                    let mut #TEMP = #options_var.clone();
                    #TEMP.offset = #offset;
                    #TEMP
                };
                #value
            };
        }

        self
    }

    fn wrap_condition(mut self) -> Self {
        if self.0.if_cond.is_some() {
            let ident = &self.0.ident;
            let value = &self.1;
            self.1 = quote! {
                if let Some(#ident) = #ident.as_mut() {
                    #value
                }
            };
        }

        self
    }
}

struct FieldGenerator<'field>(&'field StructField, TokenStream);

impl <'field> FieldGenerator<'field> {
    fn new(field: &'field StructField) -> Self {
        Self(field, TokenStream::new())
    }

    fn append_assertions(mut self) -> Self {
        let assertions = get_assertions(&self.0.assert);
        let value = &self.1;
        self.1 = quote! {
            #value
            #(#assertions)*
        };

        self
    }

    fn assign_to_var(mut self) -> Self {
        let value = &self.1;
        self.1 = if self.0.ignore {
            quote! { let _: () = #value; }
        } else {
            let ident = &self.0.ident;
            let ty = &self.0.ty;
            quote! { let mut #ident: #ty = #value; }
        };

        self
    }

    fn deref_now(mut self, options_var: &Ident, args_var: &Ident) -> Self {
        if !self.0.deref_now && !self.0.postprocess_now {
            return self;
        }

        if let Some(after_parse) = get_after_parse_handler(&self.0) {
            let after_parse = AfterParseGenerator::new(self.0)
                .get_temp_value()
                .call_after_parse(after_parse, options_var, args_var)
                .finish();

            let value = &self.1;
            self.1 = quote! {{
                let mut #TEMP = #value;
                #after_parse
                #TEMP
            }};
        }

        self
    }

    fn finish(self) -> TokenStream {
        self.1
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

        let ty = &self.0.ty;
        let value = &self.1;

        self.1 = match &self.0.map {
            Map::None => return self,
            Map::Map(map) => {
                quote! {{
                    #coerce_fn
                    (__binread_coerce::<#ty, _, _>(#map))(#value)
                }}
            },
            Map::Try(try_map) => {
                // TODO: Position should always just be saved once for a field if used
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
        let args = get_passed_args(&self.0.args);
        let options = get_read_options_override_keys(get_name_option_pairs_ident_expr(self.0));
        let value = &self.1;
        self.1 = quote! {
            let #args_var = #args;
            let #options_var = #options;
            #value
        };

        self
    }

    fn read_value(mut self, options_var: &Ident, args_var: &Ident) -> Self {
        // TODO: Parser needs to ensure invalid combinations of properties are
        // not used. ignore, default, parse_with, calc = cannot be combined!
        self.1 = if self.0.ignore {
            quote! { () }
        } else if self.0.default {
            quote! { <_>::default() }
        } else if let Some(ref expr) = self.0.calc {
            quote! { #expr }
        } else {
            let read_method = if let Some(parser) = &self.0.parse_with {
                parser.clone()
            } else {
                quote! { #READ_METHOD }
            };

            quote! {
                #read_method(#READER, #options_var, #args_var.clone())
            }
        };

        self
    }

    fn try_conversion(mut self) -> Self {
        let result = &self.1;
        // TODO: Collapse these conditions into Field struct
        if self.0.ignore || self.0.default || self.0.calc.is_some() {
            if self.0.do_try {
                self.1 = quote! { Some(#result) };
            }
        } else {
            self.1 = if self.0.do_try {
                quote! { #result.ok() }
            } else {
                let handle_error = debug_template::handle_error();
                quote! { #result#handle_error? }
            };
        }

        self
    }

    fn wrap_condition(mut self) -> Self {
        if let Some(cond) = &self.0.if_cond {
            let value = &self.1;
            self.1 = quote! {
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
        if self.0.restore_position {
            self.1 = wrap_save_restore(self.1);
        }

        self
    }

    fn wrap_seek(mut self) -> Self {
        let seek_before = generate_seek_before(self.0);
        let seek_after = generate_seek_after(self.0);
        if !seek_before.is_empty() || !seek_after.is_empty() {
            let value = &self.1;
            self.1 = quote! {{
                #seek_before
                let #TEMP = #value;
                #seek_after
                #TEMP
            }};
        }

        self
    }
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
    let skip_after_parse =
        field.map.is_some() || field.ignore || field.default ||
        field.calc.is_some() || field.parse_with.is_some();

    if skip_after_parse {
        None
    } else if field.do_try {
        Some(AFTER_PARSE_TRY)
    } else {
        Some(AFTER_PARSE)
    }
}

ident_str! {
    VARIABLE_NAME = "variable_name";
    COUNT = "count";
    OFFSET = "offset";
}

fn get_name_option_pairs_ident_expr(field: &StructField) -> impl Iterator<Item = (IdentStr, TokenStream)> {
    let endian = get_endian_tokens(&field.endian);

    let offset = field.offset.as_ref().map(|offset| (OFFSET, offset.clone()));

    let variable_name = if cfg!(feature = "debug_template") {
        let name = field.ident.to_string();
        Some((VARIABLE_NAME, quote!{ Some(#name) }))
    } else {
        None
    };

    let count = field.count.as_ref().map(|count| (COUNT, quote!{ Some((#count) as usize) }));

    count.into_iter()
        .chain(endian)
        .chain(variable_name)
        .chain(offset)
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
