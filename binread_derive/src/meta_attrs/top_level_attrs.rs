use super::*;
use super::parser::{TopLevelAttr, MetaAttrList, BinreadAttribute, MetaList, MetaLit};
use syn::spanned::Spanned;
use syn::parse::Parse;
use proc_macro2::Span;
use crate::CompileError;
use quote::ToTokens;
use super::parser::ImportArg;
use crate::meta_attrs::parser::ImportArgTuple;

#[derive(Debug, Clone)]
pub struct TopLevelAttrs {
    // ======================
    //  Top-Only Attributes
    // ======================
    pub import: Imports, // Vec<Ident>, Vec<Type>
    pub return_all_errors: SpannedValue<bool>,
    pub return_unexpected_error: SpannedValue<bool>,

    // ======================
    //  All-level attributes
    // ======================
    // endian
    pub little: SpannedValue<bool>,
    pub big: SpannedValue<bool>,
    
    // assertions/error handling
    pub assert: Vec<Assert>,
    pub magic: Option<TokenStream>,
    pub magic_type: Option<MagicType>,
    pub pre_assert: Vec<Assert>,
}

macro_rules! get_tla_type {
    ($tla:ident.$variant:ident) => {
        $tla.iter()
            .filter_map(|x|{
                if let TopLevelAttr::$variant(x) = x {
                    Some(x)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
}

impl TopLevelAttrs {
    pub fn finalize(self) -> Result<Self, SpanError> {
        if *self.big && *self.little {
            SpanError::err(
                self.big.span().join(self.little.span()).unwrap(),
                "Cannot set endian to both big and little endian"
            )?;
        }

        Ok(self)
    }

    pub fn from_derive_input(input: &syn::DeriveInput) -> Result<Self, CompileError> { 
        Self::from_attrs(&input.attrs)
    }

    pub fn from_variant(input: &syn::Variant) -> Result<Self, CompileError> {
        Self::from_attrs(&input.attrs)
    }

    pub fn from_attrs(attrs: &Vec<syn::Attribute>) -> Result<Self, CompileError> {
        let attrs: Vec<TopLevelAttr> =
            attrs
                .iter()
                .filter(|x| x.path.is_ident("br") || x.path.is_ident("binread"))
                .map(tlas_from_attribute)
                .collect::<Result<Vec<TlaList>, CompileError>>()?
                .into_iter()
                .map(|x| x.0.into_iter())
                .flatten()
                .collect();

        Self::from_top_level_attrs(attrs)
    }

    pub fn from_top_level_attrs(attrs: Vec<TopLevelAttr>) -> Result<Self, CompileError> {
        let bigs = get_tla_type!(attrs.Big);
        let littles = get_tla_type!(attrs.Little);

        if bigs.len() + littles.len() > 1 {
            join_spans_err(&bigs, &littles, "Cannot set endianess more than once")?;
        }

        let return_all_errors = get_tla_type!(attrs.ReturnAllErrors);
        let return_unexpected_errors = get_tla_type!(attrs.ReturnUnexpectedError);

        if return_all_errors.len() + return_unexpected_errors.len() > 1 {
            join_spans_err(&bigs, &littles, "Cannot set more than one return type")?;
        }

        let magics = get_tla_type!(attrs.Magic);
        let imports = get_tla_type!(attrs.Import);
        let import_tuples = get_tla_type!(attrs.ImportTuple);
        let asserts = get_tla_type!(attrs.Assert);
        let pre_asserts = get_tla_type!(attrs.PreAssert);

        let magic = get_only_first(&magics, "Cannot define multiple magic values")?;

        // TODO: this is basically get_only_first, but for incompatible attributes
        if imports.len() > 0 && import_tuples.len() > 0 {
            let mut spans = imports.iter()
                .map(Spanned::span)
                .chain(import_tuples.iter().map(Spanned::span));

            let first = spans.next().unwrap();
            let span = spans.fold(first, |x, y| x.join(y).unwrap());

            return Err(CompileError::SpanError(SpanError::new(
                span,
                "Cannot mix import and import_tuple"
            )));
        }

        let import = get_only_first(&imports, "Cannot define multiple sets of arguments")?;
        let import_tuple = get_only_first(&import_tuples, "Cannot define multiple sets of tuple arguments")?;

        Ok(Self {
            assert: asserts.into_iter().map(convert_assert).collect::<Result<_, _>>()?,
            big: first_span_true(bigs),
            little: first_span_true(littles),
            magic: magic.map(magic_to_tokens),
            magic_type: magic.map(magic_to_type),
            import: convert_import(import, import_tuple).unwrap_or_default(),
            return_all_errors: first_span_true(return_all_errors),
            return_unexpected_error: first_span_true(return_unexpected_errors),
            pre_assert: pre_asserts.into_iter().map(convert_assert).collect::<Result<_, _>>()?,
        })
    }
}

fn magic_to_type(magic: &&MetaLit<impl syn::parse::Parse>) -> MagicType {
    let magic = &magic.lit;
    match magic {
        Lit::Str(_) => MagicType::Str,
        Lit::ByteStr(_) => MagicType::ByteStr,
        Lit::Byte(_) => MagicType::Byte,
        Lit::Char(_) => MagicType::Char,
        Lit::Int(i) => MagicType::Int(i.suffix().to_owned()),
        Lit::Float(_) => MagicType::Float,
        Lit::Bool(_) => MagicType::Bool,
        Lit::Verbatim(_) => MagicType::Verbatim
    }
}

fn magic_to_tokens(magic: &&MetaLit<impl syn::parse::Parse>) -> TokenStream {
    let magic = &magic.lit;
    if let Lit::Str(_) | Lit::ByteStr(_) = magic {
        quote::quote!{
            *#magic
        }
    } else {
        magic.to_token_stream()
    }
}

fn convert_import<K: Parse>(import: Option<&&MetaList<K, ImportArg>>, import_tuple: Option<&&ImportArgTuple>) -> Option<Imports> {
    if let Some(tuple) = import_tuple {
        Some(Imports::Tuple(tuple.arg.ident.clone(), tuple.arg.ty.clone()))
    } else if let Some(list) = import {
        let (idents, tys): (Vec<_>, Vec<_>) =
            list.fields
                .iter()
                .cloned()
                .map(|import_arg| (import_arg.ident, import_arg.ty))
                .unzip();

        Some(Imports::List(idents, tys))
    } else {
        None
    }
}

fn get_only_first<'a, S: Spanned>(list: &'a Vec<S>, msg: &str) -> Result<Option<&'a S>, CompileError> {
    if list.len() > 1 {
        let mut spans = list.iter().map(Spanned::span);

        let first = spans.next().unwrap();
        let span = spans.fold(first, |x, y| x.join(y).unwrap());

        return Err(CompileError::SpanError(SpanError::new(
            span,
            msg
        )));
    }
    
    Ok(list.get(0))
}

fn first_span_true<S: Spanned>(vals: Vec<S>) -> SpannedValue<bool> {
    if let Some(val) = vals.get(0) {
        SpannedValue::new(
            true,
            val.span()
        )
    } else {
        Default::default()
    }
}

fn join_spans_err<S1, S2>(s1: &Vec<S1>, s2: &Vec<S2>, msg: &str) -> Result<(), CompileError>
    where S1: Spanned,
          S2: Spanned,
{
    let mut spans =
        s1.iter().map(Spanned::span)
            .chain(s2.iter().map(Spanned::span));

    let first = spans.next().unwrap();
    let span = spans.fold(first, |x, y| x.join(y).unwrap());

    Err(CompileError::SpanError(SpanError::new(
        span,
        msg
    )))
}

fn convert_assert<K>(assert: &MetaList<K, Expr>) -> Result<Assert, CompileError>
    where K: Parse + Spanned,
{
    let (cond, err) = match assert.fields[..] {
        [ref cond] => {
            (cond, None)
        }
        [ref cond, ref err] => {
            (cond, Some(err))
        }
        _ => return SpanError::err(
            assert.ident.span(),
            ""
        ).map_err(Into::into),
    };

    Ok(Assert(
        cond.into_token_stream(),
        err.map(ToTokens::into_token_stream)
    ))
}

type TlaList = MetaAttrList<TopLevelAttr>;

fn tlas_from_attribute(attr: &syn::Attribute) -> Result<TlaList, CompileError> {
    Ok(syn::parse2(attr.tokens.clone())?)
}
