use super::super::{
    keywords as kw,
    meta_types::{
        IdentPatType, IdentTypeMaybeDefault, MetaEnclosedList, MetaExpr, MetaList, MetaLit,
        MetaType, MetaValue, MetaVoid,
    },
    KeywordToken,
};
use proc_macro2::{Span, TokenStream};
use syn::{
    parse::{Parse, ParseBuffer},
    Expr,
};

pub struct WriteOnlyAttr<T>(pub T);

impl<T: KeywordToken> KeywordToken for WriteOnlyAttr<T> {
    type Token = T::Token;

    fn keyword_span(&self) -> Span {
        T::keyword_span(&self.0)
    }
}

impl<T: Parse> Parse for WriteOnlyAttr<T> {
    fn parse(buf: &ParseBuffer<'_>) -> Result<Self, syn::Error> {
        T::parse(buf).map(|x| WriteOnlyAttr(x))
    }
}

pub(crate) type AlignAfter = MetaExpr<kw::align_after>;
pub(crate) type AlignBefore = MetaExpr<kw::align_before>;
pub(crate) type Args = MetaEnclosedList<kw::args, Expr, TokenStream>;
pub(crate) type ArgsRaw = MetaExpr<kw::args_raw>;
pub(crate) type AssertLike<K> = MetaList<K, Expr>;
pub(crate) type Assert = AssertLike<kw::assert>;
pub(crate) type Big = MetaVoid<kw::big>;
pub(crate) type Calc = MetaExpr<kw::calc>;
pub(crate) type Count = MetaExpr<kw::count>;
pub(crate) type Ignore = MetaVoid<kw::ignore>;
pub(crate) type Import =
    WriteOnlyAttr<MetaEnclosedList<kw::import, IdentPatType, IdentTypeMaybeDefault>>;
pub(crate) type ImportRaw = MetaValue<kw::import_raw, IdentPatType>;
pub(crate) type IsBig = MetaExpr<kw::is_big>;
pub(crate) type IsLittle = MetaExpr<kw::is_little>;
pub(crate) type Little = MetaVoid<kw::little>;
pub(crate) type Magic = MetaLit<kw::magic>;
pub(crate) type Map = MetaExpr<kw::map>;
pub(crate) type PadAfter = MetaExpr<kw::pad_after>;
pub(crate) type PadBefore = MetaExpr<kw::pad_before>;
pub(crate) type PadSizeTo = MetaExpr<kw::pad_size_to>;
pub(crate) type WriteWith = MetaExpr<kw::write_with>;
pub(crate) type PreAssert = AssertLike<kw::pre_assert>;
pub(crate) type Repr = MetaType<kw::repr>;
pub(crate) type RestorePosition = MetaVoid<kw::restore_position>;
pub(crate) type ReturnAllErrors = MetaVoid<kw::return_all_errors>;
pub(crate) type ReturnUnexpectedError = MetaVoid<kw::return_unexpected_error>;
pub(crate) type SeekBefore = MetaExpr<kw::seek_before>;
pub(crate) type TryMap = MetaExpr<kw::try_map>;
