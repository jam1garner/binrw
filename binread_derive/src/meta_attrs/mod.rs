mod parser;
mod spanned_value;
mod top_level_attrs;
mod field_level_attrs;
pub(crate) use top_level_attrs::TopLevelAttrs;
pub(crate) use field_level_attrs::FieldLevelAttrs;
pub(crate) use spanned_value::SpannedValue;

use proc_macro2::TokenStream;
use crate::compiler_error::{CompileError, SpanError};
use syn::{Expr, Ident, Lit, parse::Parse, Type, spanned::Spanned};
use quote::ToTokens;
use proc_macro2::TokenStream as TokenStream2;

use self::parser::MetaList;

#[derive(Debug, Clone)]
pub struct Assert(pub TokenStream, pub Option<TokenStream>);

#[derive(Debug)]
struct MultiformExpr(TokenStream);

#[derive(Debug, Default, Clone)]
pub struct PassedValues(Vec<TokenStream>);

#[derive(Debug, Clone)]
pub enum PassedArgs {
    List(PassedValues),
    Tuple(TokenStream)
}

impl Default for PassedArgs {
    fn default() -> Self {
        PassedArgs::List(PassedValues::default())
    }
}

impl PassedValues {
    pub fn iter(&self) -> impl Iterator<Item = &TokenStream> {
        self.0.iter()
    }
}

#[derive(Debug, Clone)]
pub enum Imports {
    List(Vec<Ident>, Vec<Type>),
    Tuple(Ident, Box<Type>)
}

impl Default for Imports {
    fn default() -> Self {
        Imports::List(Vec::new(), Vec::new())
    }
}

impl Imports {
    pub fn idents(&self) -> TokenStream2 {
        match self {
            Imports::List(idents, _) => {
                let idents = idents.iter();
                quote::quote! {
                    (#(mut #idents,)*)
                }
            },
            Imports::Tuple(ident, _) => quote::quote! {
                mut #ident
            }
        }
    }

    pub fn types(&self) -> TokenStream2 {
        match self {
            Imports::List(_, types) => {
                let types = types.iter();
                quote::quote! {
                    (#(#types,)*)
                }
            },
            Imports::Tuple(_, ty) => {
                ty.to_token_stream()
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Imports::List(idents, _) => idents.is_empty(),
            Imports::Tuple(_, _) => false
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum MagicType {
    Str,
    ByteStr,
    Byte,
    Char,
    Int(String),
    Float,
    Bool,
    Verbatim
}

fn check_mutually_exclusive<'a, S1, S2, Iter1, Iter2>(a: Iter1, b: Iter2, msg: impl Into<String>) -> Result<(), CompileError>
    where S1: Spanned + 'a,
          S2: Spanned + 'a,
          Iter1: Iterator<Item = &'a S1>,
          Iter2: Iterator<Item = &'a S2>,
{
    let mut a = a.peekable();
    let mut b = b.peekable();
    if a.peek().is_some() && b.peek().is_some() {
        let mut spans = a.map(Spanned::span).chain(b.map(Spanned::span));
        let first = spans.next().unwrap();
        let span = spans.fold(first, |x, y| x.join(y).unwrap());

        Err(CompileError::SpanError(SpanError::new(
            span,
            msg
        )))
    } else {
        Ok(())
    }
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

fn first_span_true(mut vals: impl Iterator<Item = impl Spanned>) -> SpannedValue<bool> {
    if let Some(val) = vals.next() {
        SpannedValue::new(
            true,
            val.span()
        )
    } else {
        Default::default()
    }
}

fn get_only_first<'a, S: Spanned>(list: impl Iterator<Item = &'a S>, msg: impl Into<String>) -> Result<Option<&'a S>, CompileError> {
    let mut list = list.peekable();
    let first = list.next();

    if list.peek().is_none() {
        Ok(first)
    } else {
        let span = list.map(Spanned::span).fold(Spanned::span(first.unwrap()), |x, y| x.join(y).unwrap());
        Err(CompileError::SpanError(SpanError::new(
            span,
            msg
        )))
    }
}
