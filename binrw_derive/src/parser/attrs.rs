use super::{
    keywords as kw,
    meta_types::{IdentPatType, MetaExpr, MetaList, MetaLit, MetaType, MetaValue},
};
use syn::{Expr, Token};

pub(crate) type AlignAfter = MetaExpr<kw::align_after>;
pub(crate) type AlignBefore = MetaExpr<kw::align_before>;
pub(crate) type Args = MetaList<kw::args, Expr>;
pub(crate) type ArgsTuple = MetaExpr<kw::args_tuple>;
pub(crate) type AssertLike<K> = MetaList<K, Expr>;
pub(crate) type Assert = AssertLike<kw::assert>;
pub(crate) type Big = kw::big;
pub(crate) type Calc = MetaExpr<kw::calc>;
pub(crate) type Count = MetaExpr<kw::count>;
pub(crate) type Default = kw::default;
pub(crate) type DerefNow = kw::deref_now;
pub(crate) type If = MetaList<Token![if], Expr>;
pub(crate) type Ignore = kw::ignore;
pub(crate) type Import = MetaList<kw::import, IdentPatType>;
pub(crate) type ImportTuple = MetaValue<kw::import_tuple, IdentPatType>;
pub(crate) type IsBig = MetaExpr<kw::is_big>;
pub(crate) type IsLittle = MetaExpr<kw::is_little>;
pub(crate) type Little = kw::little;
pub(crate) type Magic = MetaLit<kw::magic>;
pub(crate) type Map = MetaExpr<kw::map>;
pub(crate) type Offset = MetaExpr<kw::offset>;
pub(crate) type OffsetAfter = MetaExpr<kw::offset_after>;
pub(crate) type PadAfter = MetaExpr<kw::pad_after>;
pub(crate) type PadBefore = MetaExpr<kw::pad_before>;
pub(crate) type PadSizeTo = MetaExpr<kw::pad_size_to>;
pub(crate) type ParseWith = MetaExpr<kw::parse_with>;
pub(crate) type PostProcessNow = kw::postprocess_now;
pub(crate) type PreAssert = AssertLike<kw::pre_assert>;
pub(crate) type Repr = MetaType<kw::repr>;
pub(crate) type RestorePosition = kw::restore_position;
pub(crate) type ReturnAllErrors = kw::return_all_errors;
pub(crate) type ReturnUnexpectedError = kw::return_unexpected_error;
pub(crate) type SeekBefore = MetaExpr<kw::seek_before>;
pub(crate) type Temp = kw::temp;
pub(crate) type Try = Token![try];
pub(crate) type TryMap = MetaExpr<kw::try_map>;
