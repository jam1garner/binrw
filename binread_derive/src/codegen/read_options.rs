use crate::{parser::{Assert, CondEndian, Endian, Enum, EnumErrorMode, EnumVariant, Input, Map, PassedArgs, Struct, StructField, UnitOnlyEnum, UnitEnumField}};
#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) fn generate(ident: &Ident, input: &Input) -> TokenStream {
    let inner = match input.map() {
        Map::None => match input {
            Input::UnitStruct(_) => generate_unit_struct(input),
            Input::Struct(s) => generate_struct(ident, input, s),
            Input::Enum(e) => generate_data_enum(e),
            Input::UnitOnlyEnum(e) => generate_unit_enum(e),
        },
        Map::Try(map) => quote! {
            #READ_METHOD(#READER, #OPT, #ARGS).and_then(#map)
        },
        Map::Map(map) => quote! {
            #READ_METHOD(#READER, #OPT, #ARGS).map(#map)
        },
    };

    quote! {
        let #POS = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))?;
        (|| {
            #inner
        })().or_else(|error| {
            #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#POS))?;
            Err(error)
        })
    }
}

fn generate_unit_struct(tla: &Input) -> TokenStream {
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

fn generate_struct(ident: &Ident, tla: &Input, ds: &Struct) -> TokenStream {
    let debug_tpl_start = get_debug_template_start(&ident);
    let read_body = generate_body(tla, &ds.fields);
    let debug_tpl_end = get_debug_template_end();
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

fn generate_unit_enum(en: &UnitOnlyEnum) -> TokenStream {
    let options = get_read_options_with_endian(&en.endian);

    match &en.repr {
        Some(repr) => generate_unit_enum_repr(&options, repr, &en.fields),
        None => generate_unit_enum_magic(&options, &en.fields),
    }
}

fn generate_unit_enum_repr(options: &TokenStream, repr: &TokenStream, variants: &[UnitEnumField]) -> TokenStream {
    let clauses = variants.iter().map(|variant| {
        let ident = &variant.ident;
        quote! {
            if #TEMP == Self::#ident as #repr {
                Ok(Self::#ident)
            }
        }
    });

    quote! {
        let #OPT = #options;
        let #TEMP: #repr = #READ_METHOD(#READER, #OPT, ())?;
        #(#clauses else)* {
            Err(#BIN_ERROR::NoVariantMatch {
                pos: #POS as _,
            })
        }
    }
}

fn generate_unit_enum_magic(options: &TokenStream, variants: &[UnitEnumField]) -> TokenStream {
    // TODO: The original code here looked as if it wanted to only handle magic
    // on variants without a pre-assert condition, but this just ended up
    // failing the generation with an early return whenever there was any
    // pre-assert condition. So not sure what is the desired behaviour here.

    let matches = variants.iter().filter_map(|field| {
        if let Some(magic) = &field.magic {
            let ident = &field.ident;
            let magic = &magic.1;
            Some(quote! { #magic => Ok(Self::#ident) })
        } else {
            None
        }
    });

    quote! {
        let #OPT = #options;
        match #READ_METHOD(#READER, #OPT, ())? {
            #(#matches,)*
            _ => {
                Err(#BIN_ERROR::NoVariantMatch {
                    pos: #POS as _
                })
            }
        }
    }
}

fn generate_data_enum(en: &Enum) -> TokenStream {
    let return_all_errors = en.error_mode != EnumErrorMode::ReturnUnexpectedError;

    let (create_error_basket, return_error) = if return_all_errors {(
        quote! {
            extern crate alloc;
            let mut #ERROR_BASKET: alloc::vec::Vec<(&'static str, #BIN_ERROR)> = alloc::vec::Vec::new();
        },
        quote! {
            Err(#BIN_ERROR::EnumErrors {
                pos: #POS as _,
                variant_errors: #ERROR_BASKET
            })
        }
    )} else {(
        TokenStream::new(),
        quote! {
            Err(#BIN_ERROR::NoVariantMatch {
                pos: #POS as _
            })
        }
    )};

    let try_each_variant = en.variants
        .iter()
        .map(|variant| {
            let body = generate_variant_impl(en, variant);

            let handle_error = if return_all_errors {
                let name = variant.ident().to_string();
                quote! {
                    #ERROR_BASKET.push((#name, #TEMP.err().unwrap()));
                }
            } else {
                TokenStream::new()
            };

            quote! {
                let #TEMP = (|| {
                    #body
                })();

                if #TEMP.is_ok() {
                    return #TEMP;
                } else {
                    #handle_error
                    #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#POS))?;
                }
            }
        });

    quote! {
        #create_error_basket
        #(#try_each_variant)*
        #return_error
    }
}

// TODO: This is distressingly close to generate_struct
fn generate_variant_impl(en: &Enum, variant: &EnumVariant) -> TokenStream {
    let (fields, assertions, return_value);
    match variant {
        EnumVariant::Variant { ident, options: ds } => {
            fields = &ds.fields[..];
            assertions = {
                let assertions = get_assertions(&en.assert).chain(get_assertions(&ds.assert));
                quote! { #(#assertions)* }
            };
            // TODO: Unit kind would be here
            let out_names = ds.iter_permanent_idents();
            return_value = if ds.is_tuple() {
                quote! { Self::#ident(#(#out_names),*) }
            } else {
                quote! { Self::#ident { #(#out_names),* } }
            };
        },
        EnumVariant::Unit(options) => {
            fields = &[];
            assertions = TokenStream::new();
            let ident = &options.ident;
            return_value = quote! { Self::#ident };
        },
    }

    // TODO: Kind of expensive since the enum is containing all the fields
    // and this is a clone.
    let tla = Input::Enum(en.with_variant(variant));
    let read_body = generate_body(&tla, &fields);
    quote! {
        #read_body
        #assertions
        Ok(#return_value)
    }
}

// TODO: replace all functions that are only passed tla with a method on TopLevelAttrs

fn generate_body(tla: &Input, field_attrs: &[StructField]) -> TokenStream {
    let arg_vars = tla.imports().idents();
    let top_level_option = get_read_options_with_endian(&tla.endian());
    let magic_handler = get_magic_pre_assertion(&tla);
    let handle_error = get_debug_template_handle_error();

    let fields = field_attrs.iter().map(|field| structs::generate_field(field));
    let after_parse = field_attrs.iter().map(|field| structs::generate_after_parse(field));

    quote! {
        let #arg_vars = #ARGS;
        let #OPT = #top_level_option;
        #magic_handler
        #(#fields)*
        let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error?;
        #(#after_parse)*
        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))#handle_error?;
    }
}

mod structs {
    use super::*;

    pub(super) fn generate_after_parse(field: &StructField) -> TokenStream {
        let handle_error = get_debug_template_handle_error();
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
        let (after_parse, deref_now) = split_by_immediate_deref(field);

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

    fn get_save_restore_positions(field: &StructField) -> (TokenStream, TokenStream) {
        if field.restore_position {
            let handle_error = get_debug_template_handle_error();
            (quote! {
                let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error?;
            }, quote! {
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))#handle_error?;
            })
        } else {
            <_>::default()
        }
    }

    pub(super) fn generate_field(field: &StructField) -> TokenStream {
        fn map_align(align: &TokenStream) -> TokenStream {
            let handle_error = get_debug_template_handle_error();
            quote! {{
                let align = (#align) as i64;
                let pos = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error? as i64;
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current((align - (pos % align)) % align))#handle_error?;
            }}
        }

        fn map_pad(pad: &TokenStream) -> TokenStream {
            let handle_error = get_debug_template_handle_error();
            quote! {
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(#pad))#handle_error?;
            }
        }

        let handle_error = get_debug_template_handle_error();
        let (save_position, restore_position) = get_save_restore_positions(&field);
        let args_var = make_ident(&field.ident, "args");
        let options_var = make_ident(&field.ident, "options");
        let args = get_passed_args(&field.args);
        let options = get_read_options_override_keys(get_name_option_pairs_ident_expr(field));
        let (cond, alternate, some) = possible_if_else(field);
        let field_var = if field.ignore {
            quote! { _: () }
        } else {
            let ident = &field.ident;
            let ty = &field.ty;
            quote! { mut #ident: #ty }
        };
        let seek_before = field.seek_before.as_ref().map(|seek| quote! {
            #SEEK_TRAIT::seek(#READER, #seek)#handle_error?;
        });
        let pad_before = field.pad_before.as_ref().map(map_pad);
        let align_before = field.align_before.as_ref().map(map_align);
        let pad_size_to_before = if field.pad_size_to.is_some() {
            quote! {
                let #POS = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error?;
            }
        } else {
            <_>::default()
        };
        let pad_size_to = field.pad_size_to.as_ref().map(|pad| quote! {{
            let pad = (#pad) as i64;
            let size = (#SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error? - #POS) as i64;
            if size < pad {
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(pad - size))#handle_error?;
            }
        }});
        let pad_after = field.pad_after.as_ref().map(map_pad);
        let align_after = field.align_after.as_ref().map(map_align);
        let try_conversion = if field.do_try {
            Some(TRY_CONVERSION)
        } else {
            None
        };
        let read_method = get_read_method(field);
        let (after_parse, deref_now) = split_by_immediate_deref(field);
        let map = get_maps(field);
        let asserts = get_field_assertions(field);

        quote! {
            #save_position
            let #args_var = #args;
            let #options_var = #options;
            let #field_var = #cond {
                #seek_before
                #pad_before
                #align_before
                #pad_size_to_before
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
                #pad_size_to
                #pad_after
                #align_after

                #TEMP
            } #alternate;
            #(#asserts)*
            #restore_position
        }
    }
}

fn get_passed_args(args: &PassedArgs) -> TokenStream {
    match args {
        PassedArgs::List(list) => quote! { (#(#list,)*) },
        PassedArgs::Tuple(tuple) => tuple.clone(),
        PassedArgs::None => quote! { () },
    }
}

ident_str! {
    VARIABLE_NAME = "variable_name";
    ENDIAN = "endian";
    COUNT = "count";
    OFFSET = "offset";
}

fn get_endian_tokens(endian: &CondEndian) -> Option<(IdentStr, TokenStream)> {
    match endian {
        CondEndian::Inherited => None,
        CondEndian::Fixed(Endian::Big) => Some((ENDIAN, quote! { #ENDIAN_ENUM::Big })),
        CondEndian::Fixed(Endian::Little) => Some((ENDIAN, quote! { #ENDIAN_ENUM::Little })),
        CondEndian::Cond(endian, condition) => {
            let (true_cond, false_cond) = match endian {
                Endian::Big => (quote!{ #ENDIAN_ENUM::Big }, quote!{ #ENDIAN_ENUM::Little }),
                Endian::Little => (quote!{ #ENDIAN_ENUM::Little }, quote!{ #ENDIAN_ENUM::Big }),
            };

            Some((ENDIAN, quote! {
                if (#condition) {
                    #true_cond
                } else {
                    #false_cond
                }
            }))
        }
    }
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

fn get_read_options_override_keys(options: impl Iterator<Item = (IdentStr, TokenStream)>) -> TokenStream {
    let mut set_options = options.map(|(key, value)| {
        quote! {
            #TEMP.#key = #value;
        }
    }).peekable();

    if set_options.peek().is_none() {
        quote! { #OPT }
    } else {
        quote! {
            &{
                let mut #TEMP = #OPT.clone();
                #(#set_options)*
                #TEMP
            }
        }
    }
}

fn get_read_options_with_endian(endian: &CondEndian) -> TokenStream {
    get_read_options_override_keys(get_endian_tokens(endian).into_iter())
}

fn get_magic_pre_assertion(tla: &Input) -> TokenStream {
    let handle_error = get_debug_template_handle_error();
    let magic = tla.magic()
        .as_ref()
        .map(|magic|{
            let (_, ref magic) = **magic;
            quote!{
                #ASSERT_MAGIC(#READER, #magic, #OPT)#handle_error?;
            }
        });
    let pre_asserts = get_assertions(&tla.pre_assert());

    quote! {
        #magic
        #(#pre_asserts)*
    }
}


fn get_assertions(asserts: &[Assert]) -> impl Iterator<Item = TokenStream> + '_ {
    asserts
        .iter()
        .map(|Assert(assert, error)| {
            let handle_error = get_debug_template_handle_error();
            let error = error.as_ref().map_or_else(
                || quote!{{
                    let mut x = Some(||{});
                    x = None;
                    x
                }},
                |err|{
                    quote!{Some(
                        || { #err }
                    )}
                });
            let assert_string = assert.to_string();

            quote!{
                #ASSERT(#READER, #assert, #assert_string, #error)#handle_error?;
            }
        })
}

fn get_field_assertions(field: &StructField) -> impl Iterator<Item = TokenStream> + '_ {
    let handle_error = get_debug_template_handle_error();
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

fn get_debug_template_handle_error() -> TokenStream {
    if cfg!(feature = "debug_template") {
        let write_end_struct = get_debug_template_end();
        quote! {
            .map_err(|e| {
                #WRITE_COMMENT(&format!("Error: {:?}", e));
                #write_end_struct
                e
            })
        }
    } else {
        TokenStream::new()
    }
}

fn get_debug_template_start(struct_name: &Ident) -> TokenStream {
    if cfg!(feature = "debug_template") {
        let struct_name = struct_name.to_string();
        quote! {
            #WRITE_START_STRUCT(#struct_name);
        }
    } else {
        TokenStream::new()
    }
}

fn get_debug_template_end() -> TokenStream {
    if cfg!(feature = "debug_template") {
        quote!{
            #WRITE_END_STRUCT (#OPT.variable_name);
        }
    } else {
        TokenStream::new()
    }
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
    let dont_after_parse =
        field.map.is_some() || field.ignore || field.default ||
        field.calc.is_some() || field.parse_with.is_some();
    if dont_after_parse {
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
            quote!{&mut},
            quote!{}
        )
    }
}

fn split_by_immediate_deref(field: &StructField) -> (IdentStr, IdentStr) {
    let after_parse = get_after_parse_handler(field);
    if field.deref_now || field.postprocess_now {
        (AFTER_PARSE_NOP, after_parse)
    } else {
        (after_parse, AFTER_PARSE_NOP)
    }
}
