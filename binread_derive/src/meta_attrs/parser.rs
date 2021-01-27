#[cfg(test)]
mod parsing_tests;
pub(crate) mod meta_types;
mod keywords;

use keywords as kw;

#[macro_use]
pub(crate) mod parse_any;

pub(crate) use meta_types::*;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, token, Ident, Token, Path, Expr};
use syn::ExprClosure;
use syn::punctuated::Punctuated;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;

// import, return_all_errors, return_unexpected_error, little, big, assert,
// magic, pre_assert, repr
parse_any! {
    enum TopLevelAttr {
        // bool type
        Big(kw::big),
        Little(kw::little),
        ReturnAllErrors(kw::return_all_errors),
        ReturnUnexpectedError(kw::return_unexpected_error),

        // lit assignment type
        Magic(MetaLit<kw::magic>),

        // ty assignment type
        Repr(Box<MetaType<kw::repr>>),

        // args type
        Import(MetaList<kw::import, IdentPatType>),
        ImportTuple(Box<ImportArgTuple>),
        Assert(MetaList<kw::assert, Expr>),
        PreAssert(MetaList<kw::pre_assert, Expr>),
        Map(MetaFunc<kw::map>),
    }
}

// args, map, ignore, default, calc, count, offset, if_cond, deref_now,
// postprocess_now, restore_position, little, big, is_big, is_little,
// assert, magic, pad_before, pad_after, align_before, align_after, seek_before,
// pad_size_to, parse_with

parse_any! {
    enum FieldLevelAttr {
        // bool type
        Big(kw::big),
        Little(kw::little),
        Default(kw::default),
        Ignore(kw::ignore),
        DerefNow(kw::deref_now),
        RestorePosition(kw::restore_position),
        PostProcessNow(kw::postprocess_now),
        Try(Token![try]),
        Temp(kw::temp),

        // func assignment type
        Map(MetaFunc<kw::map>),
        TryMap(MetaFunc<kw::try_map>),
        ParseWith(MetaFunc<kw::parse_with>),

        // lit assignment type
        Magic(MetaLit<kw::magic>),

        // args type
        Args(MetaList<kw::args, Expr>),
        ArgsTuple(MetaExpr<kw::args_tuple>),
        Assert(MetaList<kw::assert, Expr>),

        // expr type
        Calc(MetaExpr<kw::calc>),
        Count(MetaExpr<kw::count>),
        IsLittle(MetaExpr<kw::is_little>),
        IsBig(MetaExpr<kw::is_big>),
        Offset(MetaExpr<kw::offset>),
        OffsetAfter(MetaExpr<kw::offset_after>),
        If(MetaExpr<Token![if]>),

        PadBefore(MetaExpr<kw::pad_before>),
        PadAfter(MetaExpr<kw::pad_after>),
        AlignBefore(MetaExpr<kw::align_before>),
        AlignAfter(MetaExpr<kw::align_after>),
        SeekBefore(MetaExpr<kw::seek_before>),
        PadSizeTo(MetaExpr<kw::pad_size_to>)
    }
}

parse_any! {
    enum MetaFuncExpr {
        Path(Path),
        Closure(ExprClosure)
    }
}

impl ToTokens for MetaFuncExpr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Path(p) => p.to_tokens(tokens),
            Self::Closure(c) => c.to_tokens(tokens),
        }
    }
}

// This is like `syn::PatType` except:
// (1) Implements `Parse`;
// (2) No attributes;
// (3) Only allows an ident on the LHS instead of any `syn::Pat`.
#[derive(Debug, Clone)]
pub struct IdentPatType {
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: syn::Type
}

impl Parse for IdentPatType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(IdentPatType {
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?
        })
    }
}

#[derive(Debug, Clone)]
pub struct ImportArgTuple {
    pub ident: kw::import_tuple,
    pub parens: token::Paren,
    pub arg: IdentPatType
}

impl Parse for ImportArgTuple {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let content;
        let parens = parenthesized!(content in input);
        Ok(ImportArgTuple {
            ident,
            parens,
            arg: content.parse()?
        })
    }
}

pub(crate) struct MetaAttrList<P: Parse>(pub Vec<P>);

impl<P: Parse> Parse for MetaAttrList<P> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(MetaAttrList(
            Punctuated::<P, Token![,]>::parse_terminated(&content)?.into_iter().collect()
        ))
    }
}
