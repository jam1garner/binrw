use super::super::{
    keywords as kw,
    meta_types::{
        IdentPatType, IdentTypeMaybeDefault, MetaEnclosedList, MetaExpr, MetaList, MetaLit,
        MetaType, MetaValue, MetaVoid,
    },
    KeywordToken,
};
use proc_macro2::{Span, TokenStream};
use syn::{Expr, Token};

pub struct ReadOnlyAttr<T>(pub T);

impl<T: KeywordToken> KeywordToken for crate::parser::read::attrs::ReadOnlyAttr<T> {
    type Token = T::Token;

    fn keyword_span(&self) -> Span {
        T::keyword_span(&self.0)
    }
}

impl<T: syn::parse::Parse> syn::parse::Parse for ReadOnlyAttr<T> {
    fn parse(buf: &syn::parse::ParseBuffer<'_>) -> std::result::Result<Self, syn::Error> {
        T::parse(buf).map(|x| ReadOnlyAttr(x))
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
pub(crate) type Default = MetaVoid<kw::default>;
pub(crate) type DerefNow = MetaVoid<kw::deref_now>;
pub(crate) type ErrContext = MetaList<kw::err_context, Expr>;
pub(crate) type If = MetaList<Token![if], Expr>;
pub(crate) type Ignore = MetaVoid<kw::ignore>;
pub(crate) type Import =
    ReadOnlyAttr<MetaEnclosedList<kw::import, IdentPatType, IdentTypeMaybeDefault>>;
pub(crate) type ImportRaw = MetaValue<kw::import_raw, IdentPatType>;
pub(crate) type IsBig = MetaExpr<kw::is_big>;
pub(crate) type IsLittle = MetaExpr<kw::is_little>;
pub(crate) type Little = MetaVoid<kw::little>;
pub(crate) type Magic = MetaLit<kw::magic>;
pub(crate) type Map = MetaExpr<kw::map>;
pub(crate) type Offset = MetaExpr<kw::offset>;
pub(crate) type OffsetAfter = MetaExpr<kw::offset_after>;
pub(crate) type PadAfter = MetaExpr<kw::pad_after>;
pub(crate) type PadBefore = MetaExpr<kw::pad_before>;
pub(crate) type PadSizeTo = MetaExpr<kw::pad_size_to>;
pub(crate) type ParseWith = MetaExpr<kw::parse_with>;
pub(crate) type PostProcessNow = MetaVoid<kw::postprocess_now>;
pub(crate) type PreAssert = AssertLike<kw::pre_assert>;
pub(crate) type Repr = MetaType<kw::repr>;
pub(crate) type RestorePosition = MetaVoid<kw::restore_position>;
pub(crate) type ReturnAllErrors = MetaVoid<kw::return_all_errors>;
pub(crate) type ReturnUnexpectedError = MetaVoid<kw::return_unexpected_error>;
pub(crate) type SeekBefore = MetaExpr<kw::seek_before>;
pub(crate) type Temp = MetaVoid<kw::temp>;
pub(crate) type Try = MetaVoid<Token![try]>;
pub(crate) type TryMap = MetaExpr<kw::try_map>;
