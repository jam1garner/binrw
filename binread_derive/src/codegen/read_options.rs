use std::iter;
use crate::{
    meta_attrs::{
        TopLevelAttrs,
        FieldLevelAttrs,
        Assert
    },
    codegen::sanitization::*,
    compiler_error::SpanError,
    CompileError
};
use proc_macro2::TokenStream;
use quote::{quote, format_ident, ToTokens};
use syn::{Ident, DeriveInput, Type, DataStruct, DataEnum, Expr, Field, Variant};

pub fn generate(input: &DeriveInput, tla: &TopLevelAttrs) -> Result<TokenStream, CompileError> {
    match &input.data {
        syn::Data::Struct(ds) => generate_struct(input, tla, &ds),
        syn::Data::Enum(en) => generate_enum(input, tla, &en),
        _ => todo!()
    }
}

fn generate_enum(input: &DeriveInput, tla: &TopLevelAttrs, en: &DataEnum) -> Result<TokenStream, CompileError> {
    if *tla.return_all_errors && *tla.return_unexpected_error {
        let span = tla.return_all_errors.span().join(tla.return_unexpected_error.span()).unwrap();
        SpanError::err(span, "Must only select a single enum error return type")?;
    }

    if en.variants.is_empty() {
        SpanError::err(en.brace_token.span, "Cannot construct an enum with no variants")?;
    }

    // return_all_errors is default behavior
    let return_all_errors = *tla.return_all_errors || !*tla.return_all_errors;


    let enum_name = &input.ident;
    let variant_funcs =
        en.variants.iter().map(|variant| {
                format_ident!(
                    "__binread_generated_parse_enum_{}_variant_{}",
                    enum_name,
                    variant.ident
                )
            });
    let variant_funcs2 = variant_funcs.clone();

    let variant_func_impls = en.variants
                                .iter()
                                .map(|variant| generate_variant_impl(enum_name, tla, variant))
                                .collect::<Result<Vec<_>, _>>()?;


    let variant_prototypes = iter::repeat(quote!{<R: #READ_TRAIT + #SEEK_TRAIT>(#READER: &mut R, #OPT: &#OPTIONS, #ARGS: <#enum_name as #TRAIT_NAME>::Args) -> #BIN_RESULT<#enum_name>});
    let last_attempt = IdentStr("__binread_generated_last_attempt");
    let error_basket = IdentStr("__binread_generated_error_basket");
    let last_pos = IdentStr("__binread_generated_pos_before_variant");
    let (seek_trait, reader, seek_from, opt, args) = (SEEK_TRAIT, READER, SEEK_FROM, OPT, ARGS);

    let (create_error_basket, handle_error, end_error) = if return_all_errors {
        (
            quote!{
                let mut #error_basket: Vec<(&'static str, #BIN_ERROR)> = vec![];
            },
            en.variants.iter().map(|variant|{
                let name = variant.ident.to_string();
                quote!{
                    #error_basket.push((#name, #last_attempt.err().unwrap()));
                    #seek_trait::seek(#reader, #seek_from::Start(#last_pos))?;
                }
            }).collect::<Vec<_>>(),
            quote!{
                Err(#BIN_ERROR::EnumErrors {
                    pos: #last_pos as usize,
                    variant_errors: #error_basket
                })
            }
        )
    } else {
        (
            quote!{},
            en.variants.iter().map(|_|{
                quote!{ #seek_trait::seek(#reader, #seek_from::Start(#last_pos))?; }
            }).collect(),
            quote!{
                Err(#BIN_ERROR::NoVariantMatch {
                    pos: #last_pos as usize
                })
            }
        )
    };

    Ok(quote!{
        #create_error_basket
        let #last_pos = #seek_trait::seek(#reader, #seek_from::Current(0))?;
        #(
            fn #variant_funcs#variant_prototypes {
                #variant_func_impls
            }
        )*

        #(
            let #last_attempt = #variant_funcs2(#reader, #opt, #args);
            if #last_attempt.is_ok() {
                return #last_attempt;
            } else {
                #handle_error
            }
        )*

        #end_error
    })
}

fn generate_variant_impl(enum_name: &Ident, tla: &TopLevelAttrs, variant: &Variant)
    -> Result<TokenStream, CompileError>
{
    let tla = merge_tlas(tla, TopLevelAttrs::from_variant(variant)?)?;

    let variant_name = &variant.ident;
    let (name, ty) = get_name_types_fields(variant.fields.iter());
    let field_attrs = get_field_attrs(variant.fields.iter())?;

    let body = generate_body(&tla, &field_attrs, &name, ty)?;

    let build_variant = match &variant.fields {
        syn::Fields::Named(_) => quote!{ #enum_name::#variant_name { #(#name),* } },
        syn::Fields::Unnamed(_) => quote!{ #enum_name::#variant_name (#(#name),*) },
        syn::Fields::Unit => quote!{ #enum_name::#variant_name },
    };

    Ok(quote!{
        #body

        Ok(#build_variant)
    })
}

fn merge_tlas(top_level: &TopLevelAttrs, enum_level: TopLevelAttrs) -> Result<TopLevelAttrs, CompileError> {
    let mut out = top_level.clone();

    let variant_level = enum_level.finalize()?;
    if *variant_level.big || *variant_level.little {
        out.big = variant_level.big;
        out.little = variant_level.little;
    }

    if *variant_level.return_all_errors || *variant_level.return_unexpected_error {
        SpanError::err(
            variant_level.return_unexpected_error.span()
                .join(variant_level.return_all_errors.span())
                .unwrap(),
            "Cannot specify error return type at variant level."
        )?
    }

    if !variant_level.import.0.is_empty() {
        panic!("Cannot have imports at variant level");
    }

    if variant_level.magic.is_some() {
        out.magic = variant_level.magic;
    }

    Ok(out)
}

// TODO: replace all functions that are only passed tla with a method on TopLevelAttrs

fn generate_struct(input: &DeriveInput, tla: &TopLevelAttrs, ds: &DataStruct) -> Result<TokenStream, CompileError> {
    let (field_attrs, (name, ty, struct_type))
        = (get_struct_field_attrs(&ds)?, get_struct_names_types(&ds)?);
    
    let read_struct_body = generate_body(tla, &field_attrs, &name, ty)?;
    
    let struct_name = input.ident.to_string();
    let struct_assertions = get_assertions(&tla);
 
    let write_start_struct = write_start_struct(&struct_name);
    let write_end_struct = write_end_struct();

    let build_struct = match struct_type {
        StructType::Fields => quote!{Ok(Self { #(#name),* })},
        StructType::Tuple => quote!{Ok(Self ( #(#name),* ))},
        StructType::Unit => return Ok(quote!{Ok(Self)}),
    };

    Ok(quote!{
        #write_start_struct

        #read_struct_body

        #write_end_struct

        #(
            #struct_assertions
        )*

        #build_struct
    })
}

#[allow(unused_variables)]
fn generate_body(
        tla: &TopLevelAttrs, field_attrs: &[FieldLevelAttrs], name: &Vec<Ident>, ty: Vec<&Type>
    ) -> Result<TokenStream, CompileError>
{
    let arg_vars: Vec<_> = tla.import.idents().collect();


    let count = name.len();
    let arg_vars = tla.import.idents();
    let name_args: Vec<Ident> = get_name_modified(&name, "args");
    let passed_args_closure:Vec<TokenStream> = get_passed_args(&field_attrs);
    let name_options: Vec<Ident> = get_name_modified(&name, "options");
    let new_options: Vec<_> = get_new_options(&name, &field_attrs);

    // Repeat constants
    let repeat_read_method_ident = filter_by_ignore(&field_attrs, iter::repeat(READ_METHOD));
    let repeat_options_ident = iter::repeat(OPTIONS);
    let repeat_reader_ident = iter::repeat(READER).take(count).collect::<Vec<_>>();
    let repeat_opt_ident = iter::repeat(OPT);
    let default = iter::repeat(DEFAULT);

    let possible_set_offset = get_possible_set_offset(&field_attrs, &name_options);

    let field_asserts = get_field_assertions(&field_attrs);
    let after_parse = get_after_parse_handlers(&field_attrs);
    let top_level_option = get_top_level_binread_options(&tla);
    let magic_handler = get_magic_assertion(&tla);

    let handle_error = handle_error();
    let possible_try_conversion = get_possible_try_conversion(&field_attrs);

    let repeat_handle_error = iter::repeat(&handle_error);
    let repeat_handle_error2 = iter::repeat(&handle_error);
    
    let maps = get_maps(&field_attrs);
    let names_after_ignores = ignore_names(&name, &field_attrs);
    let ty_after_ignores = ignore_types(&ty, &field_attrs);
    let opt_mut = ignore_filter(
        iter::repeat(&quote!{ mut }),
        &field_attrs,
        quote!{}
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

    Ok(quote!{
        let (#(mut #arg_vars,)*) = #ARGS;
        
        let #OPT = #top_level_option;
        
        #magic_handler

        #(
            #save_position
            #field_asserts
            let #name_args = #passed_args_closure;
            let #name_options = #new_options;
            
            #setup_possible_if
            let #opt_mut #names_after_ignores: #ty_after_ignores = 
                #possible_if {
                    #seek_before
                    #skip_before
                    #align_before
                    #pad_size_to_prep
                    let __binread__temp = #possible_some(
                        #after_parse_applier(
                            #possible_immediate_derefs,
                            #maps(#possible_try_conversion(#repeat_read_method_ident(
                                #repeat_reader_ident, #name_options, #name_args
                            ))#repeat_handle_error?),
                            #repeat_reader_ident,
                            #name_options,
                            #name_args,
                        )?
                    );

                    #pad_size_to
                    #skip_after
                    #align_after

                    __binread__temp
                } #possible_else;
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
                    #name_args,
                )#repeat_handle_error2?
            };
        )*

        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))#handle_error?;
    })
}

#[derive(Clone, Copy, Debug)]
enum StructType {
    Unit,
    Tuple,
    Fields
}

fn get_possible_set_offset(field_attrs: &[FieldLevelAttrs], name_options: &[Ident]) -> Vec<Option<TokenStream>> {
    field_attrs
        .iter()
        .zip(name_options)
        .map(|(field, name)|{
            field.offset_after
                .as_ref()
                .map(|offset|{
                    let offset = closure_wrap(offset);
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

fn get_name_types_fields<'a, I>(fields: I) -> (Vec<Ident>, Vec<&'a Type>)
    where I: IntoIterator<Item = &'a Field>,
{
    fields
        .into_iter()
        .enumerate()
        .map(|(i, field)|{
            (
                field.ident
                    .as_ref()
                    .map(Clone::clone)
                    .unwrap_or_else(|| format_ident!("self_{}", i)),
                &field.ty
            )
        })
        .unzip()
}

fn get_struct_names_types(input: &DataStruct) -> Result<(Vec<Ident>, Vec<&Type>, StructType), CompileError> {
    Ok(match input.fields {
        syn::Fields::Named(ref fields) => {
            let (names, types) = get_name_types_fields(fields.named.iter());
            (names, types, StructType::Fields)
        }
        syn::Fields::Unnamed(ref fields) => {
            let (names, types) = get_name_types_fields(fields.unnamed.iter());
            (names, types, StructType::Tuple)
        }
        syn::Fields::Unit => {
            (vec![], vec![], StructType::Unit)
        },
    })
}

fn get_name_modified(idents: &[Ident], append: &str) -> Vec<Ident> {
    idents
        .into_iter()
        .map(|ident|{
            format_ident!("__{}_binwrite_generated_{}", ident.to_string(), append)
        })
        .collect()
}

fn get_field_attrs<'a, I>(fields: I) -> Result<Vec<FieldLevelAttrs>, CompileError>
    where I: IntoIterator<Item = &'a Field>
{
    Ok(
        fields
            .into_iter()
            .map(FieldLevelAttrs::from_field)
            .collect::<Result<_, _>>()?
    )
}

fn get_struct_field_attrs(input: &DataStruct) -> Result<Vec<FieldLevelAttrs>, CompileError> {
    match input.fields {
        syn::Fields::Named(ref fields) => get_field_attrs(fields.named.iter()),
        syn::Fields::Unnamed(ref fields) => get_field_attrs(fields.unnamed.iter()),
        syn::Fields::Unit => Ok(vec![])
    }
}

fn get_passed_args(field_attrs: &[FieldLevelAttrs]) -> Vec<TokenStream> {
    field_attrs
        .into_iter()
        .map(|field_attr| {
            let passed_values: Vec<_> =
                field_attr.args
                    .iter()
                    .map(|expr|{
                        closure_wrap(expr)
                    })
                    .collect();

            quote!{
                (#(#passed_values,)*)
            }
        })
        .collect()
}

const VARIABLE_NAME: IdentStr = IdentStr("variable_name");
const ENDIAN: IdentStr = IdentStr("endian");
const COUNT: IdentStr = IdentStr("count");
const OFFSET: IdentStr = IdentStr("offset");

fn get_name_option_pairs_ident_expr(field_attrs: &FieldLevelAttrs, ident: &Ident)
    -> impl Iterator<Item = (IdentStr<'static>, TokenStream)>
{
    let endian = if let Some(condition) = &field_attrs.is_big {
        Some((ENDIAN, quote!{
            if (#condition) {
                #ENDIAN_ENUM::Big
            } else {
                #ENDIAN_ENUM::Little
            }
        }))
    } else if let Some(condition) = &field_attrs.is_little {
        Some((ENDIAN, quote!{
            if (#condition) {
                #ENDIAN_ENUM::Little
            } else {
                #ENDIAN_ENUM::Big
            }
        }))
    } else if *field_attrs.big {
        Some((ENDIAN, quote!{ #ENDIAN_ENUM::Big }))
    } else if *field_attrs.little {
        Some((ENDIAN, quote!{ #ENDIAN_ENUM::Little }))
    } else {
        None
    };
    
    let offset =
        field_attrs.offset
            .as_ref()
            .map(|offset| (OFFSET, closure_wrap(offset)));

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

fn get_modified_options<'a, I: IntoIterator<Item = (IdentStr<'a>, TokenStream)>>(option_pairs: I)
        -> TokenStream
{
    let (ident, expr): (Vec<_>, Vec<_>) = option_pairs.into_iter().unzip();
    if ident.is_empty() {
        quote!{
            #OPT
        }
    } else {
        quote!{
            &{
                let mut temp = #OPT.clone();
                
                #(
                    temp.#ident = #expr;
                )*
                
                temp
            }
        }
    }
}

fn get_new_options(idents: &[Ident], field_attrs: &[FieldLevelAttrs]) -> Vec<TokenStream> {
    field_attrs
        .into_iter()
        .zip(idents)
        .map(|(a, b)| get_modified_options(get_name_option_pairs_ident_expr(a, b)))
        .collect()
}

fn get_top_level_binread_options(tla: &TopLevelAttrs) -> TokenStream {
    let endian = if *tla.big {
        Some((ENDIAN, quote!{ #ENDIAN_ENUM::Big }))
    } else if *tla.little {
        Some((ENDIAN, quote!{ #ENDIAN_ENUM::Little }))
    } else {
        None
    };

    get_modified_options(endian.into_iter())
}

fn get_magic_assertion(tla: &TopLevelAttrs) -> Option<TokenStream> {
    let handle_error = handle_error();
    tla.magic
        .as_ref()
        .map(|magic|{
            quote!{
                #ASSERT_MAGIC(#READER, #magic, #OPT)#handle_error?;
            }
        })
}


fn get_assertions(tla: &TopLevelAttrs) -> Vec<TokenStream> {
    let handle_error = handle_error();
    tla.assert
        .iter()
        .map(|Assert(assert, error)| {
            let error = error.as_ref().map(|err|{
                    quote!{Some(
                        || { #err }
                    )}
                }).unwrap_or_else(|| quote!{{
                    let mut x = Some(||{});
                    x = None;
                    x
                }});
            let assert_string = assert.to_string();
            let assert = closure_wrap(assert);
            quote!{
                #ASSERT(#READER, #assert, #assert_string, #error)#handle_error?;
            }
        })
        .collect()
}

fn get_field_assertions(field_attrs: &[FieldLevelAttrs]) -> Vec<TokenStream> {
    let handle_error = handle_error();
    field_attrs
        .into_iter()
        .map(|field_attrs|{
            let asserts = field_attrs.assert
                .iter()
                .map(|Assert(assert, error)|{
                    let assert_string = assert.to_string();
                    let error = error.as_ref().map(|err|{
                            quote!{Some(
                                || { #err }
                            )}
                        }).unwrap_or_else(|| quote!{{
                            let mut x = Some(||{});
                            x = None;
                            x
                        }});
                    quote!{
                        #ASSERT(#READER, #assert, #assert_string, #error)#handle_error?;
                    }
                });

            quote!{#(#asserts)*}
        })
        .collect()
}

fn handle_error() -> TokenStream {
    let write_end_struct = write_end_struct();
    if cfg!(feature = "debug_template") {
        quote!{
            .map_err(|e|{
                #WRITE_COMMENT(&format!("Error: {:?}", e));
                #write_end_struct
                e
            })
        }
    } else {
        quote!{}
    }
}

fn write_start_struct(struct_name: &str) -> TokenStream {
    if cfg!(feature = "debug_template") {
        quote!{
            #WRITE_START_STRUCT (#struct_name);
        }
    } else {
        quote!{}
    }
}

fn write_end_struct() -> TokenStream {
    if cfg!(feature = "debug_template") {
        quote!{
            #WRITE_END_STRUCT (#OPT.variable_name);
        }
    } else {
        quote!{}
    }
}

fn get_maps(field_attrs: &[FieldLevelAttrs]) -> Vec<Option<TokenStream>> {
    field_attrs
        .iter()
        .map(|field_attrs| {
            field_attrs.map
                .as_ref()
                .map(|x| Some(quote!{ (#x) }))
                .unwrap_or_else(|| None)
        })
        .collect()
}


fn get_after_parse_handlers(field_attrs: &[FieldLevelAttrs]) -> Vec<&IdentStr> {
    field_attrs
        .iter()
        .map(|field_attrs| {
            let dont_after_parse = field_attrs.map.is_some() || field_attrs.ignore ||
                        field_attrs.default || field_attrs.calc.is_some()|| field_attrs.do_try ||
                        field_attrs.parse_with.is_some();
            if dont_after_parse {
                &AFTER_PARSE_NOP
            } else {
                &AFTER_PARSE
            }
        })
        .collect()
}

fn ignore_filter<T, I>(idents: I, field_attrs: &[FieldLevelAttrs], replace_filter: TokenStream) -> Vec<TokenStream>
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

fn ignore_names(idents: &[Ident], field_attrs: &[FieldLevelAttrs]) -> Vec<TokenStream> {
    ignore_filter(idents, field_attrs, quote!{ _ })
}

fn ignore_types(idents: &[&Type], field_attrs: &[FieldLevelAttrs]) -> Vec<TokenStream> {
    ignore_filter(idents, field_attrs, quote! { () })
}

fn filter_by_ignore<'a, I>(field_attrs: &[FieldLevelAttrs], idents: I) -> Vec<TokenStream>
    where I: IntoIterator<Item = IdentStr<'a>>
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

fn possible_if_else(field_attrs: &[FieldLevelAttrs], idents: &[Ident]) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
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

fn possible_if_let(field_attrs: &[FieldLevelAttrs], idents: &[Ident]) -> (Vec<TokenStream>, Vec<TokenStream>) {
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

fn generate_skips(field_attrs: &[FieldLevelAttrs]) -> Skips {
    let mut seek_before = vec![];
    let mut skip_before = vec![];
    let mut align_before = vec![];
    let mut pad_size_to_prep = vec![];
    let mut pad_size_to = vec![];
    let mut skip_after = vec![];
    let mut align_after = vec![];

    let handle_error = handle_error();
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

fn split_by_immediate_deref<'a, 'b>(after_parse: Vec<&'a IdentStr<'a>>, field_attrs: &'b [FieldLevelAttrs])
    -> (Vec<&'a IdentStr<'a>>, Vec<&'a IdentStr<'a>>)
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


fn save_restore_position(field_attrs: &[FieldLevelAttrs]) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let handle_error = handle_error();
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

const SAVED_POSITION: IdentStr = IdentStr("__binread_generated_saved_position");

fn get_possible_try_conversion(field_attrs: &[FieldLevelAttrs]) -> Vec<TokenStream> {
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