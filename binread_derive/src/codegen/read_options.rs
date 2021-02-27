use core::iter;
use crate::{parser::{Assert, CondEndian, Endian, Enum, EnumErrorMode, EnumVariant, Input, Map, PassedArgs, Struct, StructField, UnitOnlyEnum, UnitEnumField}};
#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{Ident, Type};

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

// TODO: Maybe should be Struct function, also too much cloning
fn map_fields(fields: &[StructField]) -> impl Iterator<Item = (Ident, Option<Ident>, Type)> + Clone + '_ {
    fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let ident = field.ident.clone().unwrap_or_else(|| format_ident!("self_{}", i));
            (ident.clone(), if field.temp { None } else { Some(ident) }, field.ty.clone())
        })
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
    let fields = map_fields(&ds.fields);

    // TODO: Do not collect, use iterators directly. Also, less cloning
    let in_names = fields.clone().map(|f| f.0).collect::<Vec<_>>();
    let ty = fields.clone().map(|f| f.2).collect::<Vec<_>>();
    let out_names = fields.filter_map(|f| f.1);

    let debug_tpl_start = get_debug_template_start(&ident);
    let read_body = generate_body(tla, &ds.fields, &in_names, &ty);
    let debug_tpl_end = get_debug_template_end();
    let assertions = get_assertions(&ds.assert);
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
    let todo = Vec::new();
    let (in_names, ty, fields, assertions, return_value);
    match variant {
        EnumVariant::Variant { ident, options: ds } => {
            fields = &ds.fields;
            let fields = map_fields(&ds.fields);
            // TODO: Do not collect, use iterators directly. Also, less cloning
            in_names = fields.clone().map(|f| f.0).collect::<Vec<_>>();
            ty = fields.clone().map(|f| f.2).collect::<Vec<_>>();
            assertions = {
                let assertions = get_assertions(&en.assert).chain(get_assertions(&ds.assert));
                quote! { #(#assertions)* }
            };
            // TODO: Unit kind would be here
            let out_names = fields.filter_map(|f| f.1);
            return_value = if ds.is_tuple() {
                quote! { Self::#ident(#(#out_names),*) }
            } else {
                quote! { Self::#ident { #(#out_names),* } }
            };
        },
        EnumVariant::Unit(options) => {
            fields = &todo;
            in_names = Vec::new();
            ty = Vec::new();
            assertions = TokenStream::new();
            let ident = &options.ident;
            return_value = quote! { Self::#ident };
        },
    }

    // TODO: Kind of expensive since the enum is containing all the fields
    // and this is a clone.
    let tla = Input::Enum(en.with_variant(variant));
    let read_body = generate_body(&tla, &fields, &in_names, &ty);
    quote! {
        #read_body
        #assertions
        Ok(#return_value)
    }
}

// TODO: replace all functions that are only passed tla with a method on TopLevelAttrs

fn generate_body(tla: &Input, field_attrs: &[StructField], name: &[Ident], ty: &[Type]) -> TokenStream {
    let count = name.len();
    let arg_vars = tla.imports().idents();
    let name_args: Vec<Ident> = get_name_modified(&name, "args");
    let passed_args_closure:Vec<TokenStream> = get_passed_args(&field_attrs);
    let name_options: Vec<Ident> = get_name_modified(&name, "options");
    let new_options: Vec<_> = get_new_options(&name, &field_attrs);

    // Repeat constants
    let repeat_read_method_ident = filter_by_ignore(&field_attrs, iter::repeat(READ_METHOD));
    let _repeat_options_ident = iter::repeat(OPTIONS);
    let repeat_reader_ident = iter::repeat(READER).take(count).collect::<Vec<_>>();
    let _repeat_opt_ident = iter::repeat(OPT);
    let _default = iter::repeat(DEFAULT);

    let possible_set_offset = get_possible_set_offset(&field_attrs, &name_options);

    let field_asserts = get_field_assertions(&field_attrs);
    let after_parse = get_after_parse_handlers(&field_attrs);
    let top_level_option = get_read_options_with_endian(&tla.endian());
    let magic_handler = get_magic_pre_assertion(&tla);

    let handle_error = get_debug_template_handle_error();
    let possible_try_conversion = get_possible_try_conversion(&field_attrs);

    let repeat_handle_error = iter::repeat(&handle_error);
    let repeat_handle_error2 = iter::repeat(&handle_error);

    let maps = get_maps(&field_attrs, ty);
    let names_after_ignores = ignore_names(&name, &field_attrs);
    let ty_after_ignores = ignore_types(ty, &field_attrs);
    let opt_mut = ignore_filter(
        iter::repeat(&quote!{ mut }),
        &field_attrs,
        &quote!{}
    );

    // Handle the actual conditions for if tags
    let (setup_possible_if, possible_if, possible_else, possible_some)
        = possible_if_else(&field_attrs, &name);

    // Handle option types for if statements
    let (possible_mut, possible_if_let) = possible_if_let(&field_attrs, &name);

    let Skips { seek_before, skip_before, align_before, pad_size_to_prep,
                pad_size_to, skip_after, align_after, } = generate_skips(&field_attrs);

    let (after_parse, possible_immediate_derefs)
        = split_by_immediate_deref(after_parse, &field_attrs);

    let after_parse_applier = iter::repeat(&AFTER_PARSE_IDENTITY);

    let (save_position, restore_position) = save_restore_position(&field_attrs);

    quote!{
        let #arg_vars = #ARGS;

        let #OPT = #top_level_option;

        #magic_handler

        #(
            #save_position
            let #name_args = (#passed_args_closure).clone();
            let #name_options = #new_options;

            #setup_possible_if
            let #opt_mut #names_after_ignores: #ty_after_ignores =
                #possible_if {
                    #seek_before
                    #skip_before
                    #align_before
                    #pad_size_to_prep
                    let __binread_temp = #possible_try_conversion(#repeat_read_method_ident(
                        #repeat_reader_ident, #name_options, (#name_args).clone()
                    ))#repeat_handle_error?;
                    let __binread_temp = #possible_some(
                        #after_parse_applier(
                            #possible_immediate_derefs,
                            #maps,
                            #repeat_reader_ident,
                            #name_options,
                            #name_args.clone(),
                        )?
                    );

                    #pad_size_to
                    #skip_after
                    #align_after

                    __binread_temp
                } #possible_else;
            #field_asserts
            #restore_position
        )*

        #(
            #possible_set_offset
        )*

        let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error?;

        #(
            #possible_if_let {
                #after_parse(
                    #possible_mut #name,
                    #repeat_reader_ident,
                    #name_options,
                    (#name_args).clone(),
                )#repeat_handle_error2?
            };
        )*

        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))#handle_error?;
    }
}

fn get_possible_set_offset(field_attrs: &[StructField], name_options: &[Ident]) -> Vec<Option<TokenStream>> {
    field_attrs
        .iter()
        .zip(name_options)
        .map(|(field, name)|{
            field.offset_after
                .as_ref()
                .map(|offset|{
                    quote!{
                        let #name = &{
                            let mut temp = #name.clone();
                            temp.offset = #offset;
                            temp
                        };
                    }
                })
        })
        .collect()
}

fn get_name_modified(idents: &[Ident], append: &str) -> Vec<Ident> {
    idents
        .iter()
        .map(|ident|{
            format_ident!("__{}_binread_generated_{}", ident.to_string(), append)
        })
        .collect()
}

fn get_passed_args(field_attrs: &[StructField]) -> Vec<TokenStream> {
    field_attrs
        .iter()
        .map(|field_attr| {
            match &field_attr.args {
                PassedArgs::List(list) => {
                    let passed_values: Vec<_> =
                        list.iter()
                            .map(|expr|{
                                closure_wrap(expr)
                            })
                            .collect();

                    quote!{
                        (#(#passed_values,)*)
                    }
                },
                PassedArgs::Tuple(tok) => tok.clone(),
                PassedArgs::None => quote!{ () },
            }

        })
        .collect()
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

fn get_name_option_pairs_ident_expr(field_attrs: &StructField, ident: &Ident)
    -> impl Iterator<Item = (IdentStr, TokenStream)>
{
    let endian = get_endian_tokens(&field_attrs.endian);

    let offset =
        field_attrs.offset
            .as_ref()
            .map(|offset| (OFFSET, offset.clone()));

    let variable_name = if cfg!(feature = "debug_template") {
        let name = ident.to_string();
        Some((VARIABLE_NAME, quote!{ Some(#name) }))
    } else {
        None
    };

    let count = field_attrs.count.as_ref().map(|count| (COUNT, quote!{ Some((#count) as usize) }));

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

fn get_new_options(idents: &[Ident], field_attrs: &[StructField]) -> Vec<TokenStream> {
    field_attrs
        .iter()
        .zip(idents)
        .map(|(a, b)| get_read_options_override_keys(get_name_option_pairs_ident_expr(a, b)))
        .collect()
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

fn get_field_assertions(field_attrs: &[StructField]) -> Vec<TokenStream> {
    let handle_error = get_debug_template_handle_error();
    field_attrs
        .iter()
        .map(|field_attrs|{
            let asserts = field_attrs.assert
                .iter()
                .map(|Assert(assert, error)|{
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
                    quote!{
                        #ASSERT(#READER, #assert, #assert_string, #error)#handle_error?;
                    }
                });

            quote!{#(#asserts)*}
        })
        .collect()
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

fn get_maps(field_attrs: &[StructField], types: &[Type]) -> Vec<TokenStream> {
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

    field_attrs
        .iter()
        .zip(types.iter())
        .map(|(field_attrs, ty)| {
            if let Map::Try(try_map) = &field_attrs.map {
                quote!{ {
                    let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))?;

                    #coerce_fn
                    (__binread_coerce::<::core::result::Result<#ty, _>, _, _>(#try_map))(__binread_temp).map_err(|e| {
                        #BIN_ERROR::Custom {
                            pos: #SAVED_POSITION as _,
                            err: Box::new(e) as _,
                        }
                    })?
                } }
            } else if let Map::Map(map) = &field_attrs.map {
                quote!{ {
                    #coerce_fn
                    (__binread_coerce::<#ty, _, _>(#map))(__binread_temp)
                } }
            } else {
                quote!{ __binread_temp }
            }
        })
        .collect()
}


fn get_after_parse_handlers(field_attrs: &[StructField]) -> Vec<&IdentStr> {
    field_attrs
        .iter()
        .map(|field_attrs| {
            let dont_after_parse = field_attrs.map.is_some() || field_attrs.ignore ||
                        field_attrs.default || field_attrs.calc.is_some() ||
                        field_attrs.parse_with.is_some();
            if dont_after_parse {
                &AFTER_PARSE_NOP
            } else if field_attrs.do_try {
                &AFTER_PARSE_TRY
            } else {
                &AFTER_PARSE
            }
        })
        .collect()
}

fn ignore_filter<T, I>(idents: I, field_attrs: &[StructField], replace_filter: &TokenStream) -> Vec<TokenStream>
    where T: ToTokens,
          I: IntoIterator<Item = T>
{
    idents
        .into_iter()
        .zip(field_attrs)
        .map(|(ident, field_attrs)|{
            if field_attrs.ignore {
               replace_filter.clone()
            } else {
                quote!{ #ident }
            }
        })
        .collect()
}

fn ignore_names(idents: &[Ident], field_attrs: &[StructField]) -> Vec<TokenStream> {
    ignore_filter(idents, field_attrs, &quote!{ _ })
}

fn ignore_types(idents: &[Type], field_attrs: &[StructField]) -> Vec<TokenStream> {
    ignore_filter(idents, field_attrs, &quote! { () })
}

fn filter_by_ignore<I>(field_attrs: &[StructField], idents: I) -> Vec<TokenStream>
    where I: IntoIterator<Item = IdentStr>
{
    idents
        .into_iter()
        .zip(field_attrs)
        .map(|(ident, field_attrs)|{
            if field_attrs.ignore {
                quote! { #READ_METHOD_NOP }
            } else if let Some(ref parser) = field_attrs.parse_with {
                quote! { #parser }
            } else if field_attrs.default {
                quote! { #READ_METHOD_DEFAULT }
            } else if let Some(ref expr) = field_attrs.calc {
                quote! { (|_: &mut _, _, _| -> #BIN_RESULT<_> {Ok(#expr)}) }
            } else {
                quote!{ #ident }
            }
        })
        .collect()
}

fn possible_if_else(field_attrs: &[StructField], idents: &[Ident]) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
    let (cond_eval, if_stmt) =
        field_attrs
            .iter()
            .zip(get_name_modified(idents, "cond_evaluated"))
            .map(|(field_attrs, cond_evaluated)|{
                match field_attrs.if_cond {
                    Some(ref cond) => (
                        quote!{let #cond_evaluated: bool = #cond;},
                        quote!{if (#cond_evaluated)},
                    ),
                    None => (quote!{}, quote!{})
                }

            })
            .unzip();
    let (else_stmt, somes) =
        field_attrs
            .iter()
            .map(|field_attrs|{
                if field_attrs.if_cond.is_some() {
                    (quote!{ else { None } }, quote!{ Some })
                } else {
                    (quote!{}, quote!{})
                }
            })
            .unzip();
    (
        cond_eval,
        if_stmt,
        else_stmt,
        somes
    )
}

fn possible_if_let(field_attrs: &[StructField], idents: &[Ident]) -> (Vec<TokenStream>, Vec<TokenStream>) {
    field_attrs
        .iter()
        .zip(idents)
        .map(|(field_attrs, name)|{
            if field_attrs.if_cond.is_some() {
                (
                    quote!{},
                    quote!{if let Some(#name) = #name.as_mut()}
                )
            } else {
                (
                    quote!{&mut},
                    quote!{}
                )
            }
        })
        .unzip()
}

struct Skips {
    seek_before: Vec<Option<TokenStream>>,
    skip_before: Vec<Option<TokenStream>>,
    align_before: Vec<Option<TokenStream>>,
    pad_size_to_prep: Vec<Option<TokenStream>>,
    pad_size_to: Vec<Option<TokenStream>>,
    skip_after: Vec<Option<TokenStream>>,
    align_after: Vec<Option<TokenStream>>
}

fn generate_skips(field_attrs: &[StructField]) -> Skips {
    let mut seek_before = vec![];
    let mut skip_before = vec![];
    let mut align_before = vec![];
    let mut pad_size_to_prep = vec![];
    let mut pad_size_to = vec![];
    let mut skip_after = vec![];
    let mut align_after = vec![];

    let handle_error = get_debug_template_handle_error();
    for attrs in field_attrs {
        seek_before.push(attrs.seek_before.as_ref().map(|seek|{
            let seek = closure_wrap(seek);
            quote!{
                #SEEK_TRAIT::seek(#READER, #seek)#handle_error?;
            }
        }));
        skip_before.push(attrs.pad_before.as_ref().map(|skip|{
            let skip = closure_wrap(skip);
            quote!{
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(#skip as i64))#handle_error?;
            }
        }));
        align_before.push(attrs.align_before.as_ref().map(|align|{
            let align = closure_wrap(align);
            quote!{{
                let align = #align as usize;
                let pos = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error? as usize;
                let align = ((align - (pos % align)) % align) as i64;
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(align))#handle_error?;
            }}
        }));
        pad_size_to_prep.push(attrs.pad_size_to.as_ref().map(|_|{
            quote!{
                let #POS = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error? as usize;
            }
        }));
        pad_size_to.push(attrs.pad_size_to.as_ref().map(|pad_to|{
            let pad_to = closure_wrap(pad_to);
            quote!{{
                let pad_to = #pad_to as usize;
                let #TEMP = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error? as usize;
                let size = #TEMP - #POS;
                if size < pad_to {
                    let padding = pad_to - size;
                    #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(padding as i64))#handle_error?;
                }
            }}
        }));
        skip_after.push(attrs.pad_after.as_ref().map(|skip|{
            let skip = closure_wrap(skip);
            quote!{
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(#skip as i64))#handle_error?;
            }
        }));
        align_after.push(attrs.align_after.as_ref().map(|align|{
            let align = closure_wrap(align);
            quote!{{
                let align = #align as usize;
                let pos = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error? as usize;
                let align = ((align - (pos % align)) % align) as i64;
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(align))#handle_error?;
            }}
        }));
    }

    Skips {
        seek_before,
        skip_before,
        align_before,
        pad_size_to,
        skip_after,
        align_after,
        pad_size_to_prep
    }
}

fn split_by_immediate_deref<'a, 'b>(after_parse: Vec<&'a IdentStr>, field_attrs: &'b [StructField])
    -> (Vec<&'a IdentStr>, Vec<&'a IdentStr>)
{
    after_parse
        .into_iter()
        .zip(field_attrs)
        .map(|(parser, field_attrs)|{
            if field_attrs.deref_now || field_attrs.postprocess_now {
                (&AFTER_PARSE_NOP, parser)
            } else {
                (parser, &AFTER_PARSE_NOP)
            }
        })
        .unzip()
}


fn save_restore_position(field_attrs: &[StructField]) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let handle_error = get_debug_template_handle_error();
    field_attrs
        .iter()
        .map(|field_attrs|{
            if field_attrs.restore_position {
                (
                    quote!{
                        let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))#handle_error?;
                    },
                    quote!{
                        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))#handle_error?;
                    }
                )
            } else {
                (quote!{}, quote!{})
            }
        })
        .unzip()
}

ident_str!(SAVED_POSITION = "__binread_generated_saved_position");

fn get_possible_try_conversion(field_attrs: &[StructField]) -> Vec<TokenStream> {
    field_attrs
        .iter()
        .map(|field|{
            if field.do_try {
                quote!{
                     #TRY_CONVERSION
                }
            } else {
                quote!{}
            }
        })
        .collect()
}
