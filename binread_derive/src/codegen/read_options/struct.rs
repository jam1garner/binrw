#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::{Assert, Input, Map, Struct, StructField};
use proc_macro2::TokenStream;
use quote::quote;
use super::{
    debug_template,
    get_assertions,
    get_endian_tokens,
    get_magic_pre_assertion,
    get_passed_args,
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
pub(super) fn generate_body(tla: &Input, field_attrs: &[StructField]) -> TokenStream {
    let arg_vars = tla.imports().idents();
    let options = get_read_options_with_endian(&tla.endian());
    let magic_handler = get_magic_pre_assertion(&tla);
    let handle_error = debug_template::handle_error();

    let fields = field_attrs.iter().map(|field| generate_field(field));
    let after_parse = field_attrs.iter().map(|field| generate_after_parse(field));

    quote! {
        let #arg_vars = #ARGS;
        let #OPT = #options;
        #magic_handler
        #(#fields)*
        let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error?;
        #(#after_parse)*
        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))#handle_error?;
    }
}

fn generate_after_parse(field: &StructField) -> TokenStream {
    let handle_error = debug_template::handle_error();
    let ident = &field.ident;
    // TODO: Pass as args
    let args_var = make_ident(&field.ident, "args");
    let options_var = make_ident(&field.ident, "options");
    let offset_after = field.offset_after.as_ref().map(|offset| {
        quote! {
            let #options_var = &{
                let mut #TEMP = #options_var.clone();
                #TEMP.offset = #offset;
                #TEMP
            };
        }
    });
    let (mutable, if_let) = possible_if_let(field);
    let after_parse = split_by_immediate_deref(field).0;

    quote! {
        #offset_after
        #if_let {
            #after_parse(
                #mutable #ident,
                #READER,
                #options_var,
                #args_var.clone(),
            )#handle_error?
        };
    }
}

fn save_restore_position(field: &StructField) -> (TokenStream, TokenStream) {
    if field.restore_position {
        let handle_error = debug_template::handle_error();
        (quote! {
            let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error?;
        }, quote! {
            #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))#handle_error?;
        })
    } else {
        <_>::default()
    }
}

fn generate_field(field: &StructField) -> TokenStream {
    let handle_error = debug_template::handle_error();
    let (save_position, restore_position) = save_restore_position(&field);
    let args_var = make_ident(&field.ident, "args");
    let options_var = make_ident(&field.ident, "options");
    let args = get_passed_args(&field.args);
    let options = get_read_options_override_keys(get_name_option_pairs_ident_expr(field));
    let (cond, alternate, some) = possible_if_else(field);
    let seek_before = generate_seek_before(field);
    let seek_after = generate_seek_after(field);
    let field_var = if field.ignore {
        quote! { _: () }
    } else {
        let ident = &field.ident;
        let ty = &field.ty;
        quote! { mut #ident: #ty }
    };
    let try_conversion = if field.do_try {
        Some(TRY_CONVERSION)
    } else {
        None
    };
    let read_method = get_read_method(field);
    let deref_now = split_by_immediate_deref(field).1;
    let map = get_maps(field);
    let asserts = get_field_assertions(field);

    quote! {
        #save_position
        let #args_var = #args;
        let #options_var = #options;
        let #field_var = #cond {
            #seek_before
            let #TEMP = #try_conversion(#read_method(
                #READER, #options_var, #args_var.clone()
            ))#handle_error?;
            let #TEMP = #some(
                #AFTER_PARSE_IDENTITY(
                    #deref_now,
                    #map,
                    #READER,
                    #options_var,
                    #args_var.clone(),
                )?
            );
            #seek_after
            #TEMP
        } #alternate;
        #(#asserts)*
        #restore_position
    }
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

fn get_field_assertions(field: &StructField) -> impl Iterator<Item = TokenStream> + '_ {
    let handle_error = debug_template::handle_error();
    field.assert.iter().map(move |Assert(assert, error)|{
        let assert_string = assert.to_string();
        let error = error.as_ref().map_or_else(|| quote!{{
            let mut x = Some(||{});
            x = None;
            x
        }},
        |err|{
            quote!{Some(
                || { #err }
            )}
        });
        quote! {
            #ASSERT(#READER, #assert, #assert_string, #error)#handle_error?;
        }
    })
}

fn get_maps(field: &StructField) -> TokenStream {
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

    let ty = &field.ty;

    match &field.map {
        Map::None => quote! { #TEMP },
        Map::Map(map) => {
            quote! {{
                #coerce_fn
                (__binread_coerce::<#ty, _, _>(#map))(#TEMP)
            }}
        },
        Map::Try(try_map) => {
            // TODO: Position should always just be saved once for a field if used
            quote! {{
                let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))?;

                #coerce_fn
                (__binread_coerce::<::core::result::Result<#ty, _>, _, _>(#try_map))(#TEMP).map_err(|e| {
                    #BIN_ERROR::Custom {
                        pos: #SAVED_POSITION as _,
                        err: Box::new(e) as _,
                    }
                })?
            }}
        },
    }
}

fn get_after_parse_handler(field: &StructField) -> IdentStr {
    let skip_after_parse =
        field.map.is_some() || field.ignore || field.default ||
        field.calc.is_some() || field.parse_with.is_some();

    if skip_after_parse {
        AFTER_PARSE_NOP
    } else if field.do_try {
        AFTER_PARSE_TRY
    } else {
        AFTER_PARSE
    }
}

fn get_read_method(field: &StructField) -> TokenStream {
    if field.ignore {
        quote! { #READ_METHOD_NOP }
    } else if let Some(ref parser) = field.parse_with {
        quote! { #parser }
    } else if field.default {
        quote! { #READ_METHOD_DEFAULT }
    } else if let Some(ref expr) = field.calc {
        quote! { (|_: &mut _, _, _| -> #BIN_RESULT<_> { Ok(#expr) }) }
    } else {
        quote!{ #READ_METHOD }
    }
}

fn possible_if_else(field: &StructField) -> (TokenStream, TokenStream, TokenStream) {
    if let Some(cond) = &field.if_cond {
        (quote! { if #cond }, quote! { else { None } }, quote! { Some })
    } else {
        <_>::default()
    }
}

fn possible_if_let(field: &StructField) -> (TokenStream, TokenStream) {
    if field.if_cond.is_some() {
        let ident = &field.ident;
        (<_>::default(), quote! { if let Some(#ident) = #ident.as_mut() })
    } else {
        (
            quote!{ &mut },
            <_>::default(),
        )
    }
}

// TODO: Make this less confusing
fn split_by_immediate_deref(field: &StructField) -> (IdentStr, IdentStr) {
    let after_parse = get_after_parse_handler(field);
    if field.deref_now || field.postprocess_now {
        (AFTER_PARSE_NOP, after_parse)
    } else {
        (after_parse, AFTER_PARSE_NOP)
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
