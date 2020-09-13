#[cfg(test)]
mod parsing_tests;
pub(crate) mod meta_types;
mod keywords;

use keywords as kw;

#[macro_use]
pub(crate) mod parse_any;

pub(crate) use meta_types::*;
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{parenthesized, parse_macro_input, token, Field, Ident, Token, Lit, Path, Expr};
use syn::ExprClosure;
use syn::punctuated::Punctuated;
use proc_macro2::TokenStream as TokenStream2;

// import, return_all_errors, return_unexpected_error, little, big, assert,
// magic, pre_assert
parse_any!{
    enum TopLevelAttr {
        // bool type
        Big(kw::big),
        Little(kw::little),
        ReturnAllErrors(kw::return_all_errors),
        ReturnUnexpectedError(kw::return_unexpected_error),

        // lit assignment type
        Magic(MetaLit<kw::magic>),

        // args type
        Import(MetaList<kw::import, ImportArg>),
        Assert(MetaList<kw::assert, Expr>),
        PreAssert(MetaList<kw::pre_assert, Expr>),
    }
}

// args, map, ignore, default, calc, count, offset, if_cond, deref_now,
// postprocess_now, restore_position, little, big, is_big, is_little,
// assert, magic, pad_before, pad_after, align_before, align_after, seek_before,
// pad_size_to, parse_with

parse_any!{
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
        ParseWith(MetaFunc<kw::parse_with>),

        // lit assignment type
        Magic(MetaLit<kw::magic>),

        // args type
        Args(MetaList<kw::args, Expr>),
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

parse_any!{
    enum MetaFuncExpr {
        Path(Path),
        Closure(ExprClosure)
    }
}


#[derive(Debug, Clone)]
pub struct ImportArg {
    pub ident: Ident,
    pub colon: Token![:],
    pub ty: syn::Type,
}

impl Parse for ImportArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ImportArg {
            ident: input.parse()?,
            colon: input.parse()?,
            ty: input.parse()?
        })
    }
}

pub(crate) struct MetaAttrList<P: Parse>(pub Vec<P>);

impl<P: Parse> Parse for MetaAttrList<P> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(MetaAttrList(
            Punctuated::<P, Token![,]>::parse_terminated(&content)?.into_iter().collect()
        ))
    }
}

pub(crate) struct BinreadAttribute;

impl Parse for BinreadAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(kw::br) {
            let _: kw::br = input.parse()?;
        } else {
            let _: kw::binread = input.parse()?;
        }
        Ok(Self)
    }
}
