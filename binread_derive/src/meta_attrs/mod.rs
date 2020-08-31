mod parser;
mod spanned_value;
mod top_level_attrs;
mod field_level_attrs;
pub(crate) use top_level_attrs::TopLevelAttrs;
pub(crate) use field_level_attrs::FieldLevelAttrs;
pub(crate) use spanned_value::SpannedValue;

use proc_macro2::TokenStream;
use crate::compiler_error::SpanError;
use syn::{Ident, Lit, NestedMeta, Path, Type, Field, Meta, Expr, spanned::Spanned};
use quote::ToTokens;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Assert(pub TokenStream, pub Option<TokenStream>);

#[derive(Debug)]
struct MultiformExpr(TokenStream);

#[derive(Debug, Default, Clone)]
pub struct PassedValues(Vec<TokenStream>);

impl PassedValues {
    pub fn iter(&self) -> impl Iterator<Item = &TokenStream> {
        self.0.iter()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Imports(pub Vec<Ident>, pub Vec<Type>);

impl Imports {
    pub fn idents<'a>(&'a self) -> impl Iterator<Item=&'a Ident> {
        self.0.iter()
    }
    
    pub fn types<'a>(&'a self) -> impl Iterator<Item=&'a Type> {
        self.1.iter()
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
