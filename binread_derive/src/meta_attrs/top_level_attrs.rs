use super::*;
use super::parser::{TopLevelAttr, MetaAttrList, MetaList, MetaLit};
use syn::spanned::Spanned;
use syn::parse::Parse;
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

    // other
    pub map: Option<TokenStream>,
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

    pub fn from_attrs(attrs: &[syn::Attribute]) -> Result<Self, CompileError> {
        let attrs: Vec<TopLevelAttr> =
            attrs
                .iter()
                .filter(|x| x.path.is_ident("br") || x.path.is_ident("binread"))
                .map(tlas_from_attribute)
                .collect::<Result<Vec<TlaList>, CompileError>>()?
                .into_iter()
                .flat_map(|x| x.0.into_iter())
                .collect();

        Self::from_top_level_attrs(attrs)
    }

    pub fn from_top_level_attrs(attrs: Vec<TopLevelAttr>) -> Result<Self, CompileError> {
        let bigs = get_tla_type!(attrs.Big);
        let littles = get_tla_type!(attrs.Little);

        if bigs.clone().take(2).count() + littles.clone().take(2).count() > 1 {
            return join_spans_err(bigs, littles, "Cannot set endianess more than once");
        }

        let return_all_errors = get_tla_type!(attrs.ReturnAllErrors);
        let return_unexpected_errors = get_tla_type!(attrs.ReturnUnexpectedError);

        if return_all_errors.clone().take(2).count() + return_unexpected_errors.clone().take(2).count() > 1 {
            return join_spans_err(return_all_errors, return_unexpected_errors, "Cannot set more than one return type");
        }

        let magics = get_tla_type!(attrs.Magic);
        let imports = get_tla_type!(attrs.Import);
        let import_tuples = get_tla_type!(attrs.ImportTuple);
        let asserts = get_tla_type!(attrs.Assert);
        let pre_asserts = get_tla_type!(attrs.PreAssert);
        let map = get_tla_type!(attrs.Map);

        let magic = get_only_first(magics, "Cannot define multiple magic values")?;

        check_mutually_exclusive(imports.clone(), import_tuples.clone(), "Cannot mix import and import_tuple")?;

        let import = get_only_first(imports, "Cannot define multiple sets of arguments")?;
        let import_tuple = get_only_first(import_tuples, "Cannot define multiple sets of tuple arguments")?;
        let map = get_only_first(map, "Cannot define multiple mapping functions")?;

        Ok(Self {
            assert: asserts.map(convert_assert).collect::<Result<_, _>>()?,
            big: first_span_true(bigs),
            little: first_span_true(littles),
            magic: magic.map(magic_to_tokens),
            magic_type: magic.map(magic_to_type),
            import: convert_import(import, import_tuple).unwrap_or_default(),
            return_all_errors: first_span_true(return_all_errors),
            return_unexpected_error: first_span_true(return_unexpected_errors),
            pre_assert: pre_asserts.map(convert_assert).collect::<Result<_, _>>()?,
            map: map.map(|x| x.to_token_stream()),
        })
    }
}

fn magic_to_type(magic: &MetaLit<impl syn::parse::Parse>) -> MagicType {
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

fn magic_to_tokens(magic: &MetaLit<impl syn::parse::Parse>) -> TokenStream {
    let magic = &magic.lit;
    if let Lit::Str(_) | Lit::ByteStr(_) = magic {
        quote::quote!{
            *#magic
        }
    } else {
        magic.to_token_stream()
    }
}

fn convert_import<K: Parse>(import: Option<&MetaList<K, ImportArg>>, import_tuple: Option<impl AsRef<ImportArgTuple>>) -> Option<Imports> {
    if let Some(tuple) = import_tuple {
        let tuple = tuple.as_ref();
        Some(Imports::Tuple(tuple.arg.ident.clone(), tuple.arg.ty.clone().into()))
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

fn join_spans_err<'a, Iter1, Iter2, S1, S2>(s1: Iter1, s2: Iter2, msg: impl Into<String>) -> Result<TopLevelAttrs, CompileError>
    where Iter1: Iterator<Item = &'a S1>,
          Iter2: Iterator<Item = &'a S2>,
          S1: Spanned + 'a,
          S2: Spanned + 'a,
{
    let mut spans = s1.map(Spanned::span).chain(s2.map(Spanned::span));
    let first = spans.next().unwrap();
    let span = spans.fold(first, |x, y| x.join(y).unwrap());

    Err(CompileError::SpanError(SpanError::new(
        span,
        msg
    )))
}

type TlaList = MetaAttrList<TopLevelAttr>;

fn tlas_from_attribute(attr: &syn::Attribute) -> Result<TlaList, CompileError> {
    Ok(syn::parse2(attr.tokens.clone())?)
}
