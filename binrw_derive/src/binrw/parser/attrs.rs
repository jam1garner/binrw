use super::keywords as kw;
use crate::meta_types::{
    IdentPatType, IdentTypeMaybeDefault, MetaEnclosedList, MetaExpr, MetaIdent, MetaList, MetaLit,
    MetaType, MetaValue, MetaVoid,
};
use syn::{Expr, FieldValue, Token, WherePredicate};

pub(super) type AlignAfter = MetaExpr<kw::align_after>;
pub(super) type AlignBefore = MetaExpr<kw::align_before>;
pub(super) type Args = MetaEnclosedList<kw::args, Expr, FieldValue>;
pub(super) type ArgsRaw = MetaExpr<kw::args_raw>;
pub(super) type AssertLike<Keyword> = MetaList<Keyword, Expr>;
pub(super) type Assert = AssertLike<kw::assert>;
pub(super) type Big = MetaVoid<kw::big>;
pub(super) type Bound = MetaList<kw::bound, WherePredicate>;
pub(super) type Calc = MetaExpr<kw::calc>;
pub(super) type Count = MetaExpr<kw::count>;
pub(super) type Debug = MetaVoid<kw::dbg>;
pub(super) type Default = MetaVoid<kw::default>;
pub(super) type ErrContext = MetaList<kw::err_context, Expr>;
pub(super) type If = MetaList<Token![if], Expr>;
pub(super) type Ignore = MetaVoid<kw::ignore>;
pub(super) type Import = MetaEnclosedList<kw::import, IdentPatType, IdentTypeMaybeDefault>;
pub(super) type ImportRaw = MetaValue<kw::import_raw, IdentPatType>;
pub(super) type IsBig = MetaExpr<kw::is_big>;
pub(super) type IsLittle = MetaExpr<kw::is_little>;
pub(super) type Little = MetaVoid<kw::little>;
pub(super) type Magic = MetaLit<kw::magic>;
pub(super) type Map = MetaExpr<kw::map>;
pub(super) type MapStream = MetaExpr<kw::map_stream>;
pub(super) type Offset = MetaExpr<kw::offset>;
pub(super) type PadAfter = MetaExpr<kw::pad_after>;
pub(super) type PadBefore = MetaExpr<kw::pad_before>;
pub(super) type PadSizeTo = MetaExpr<kw::pad_size_to>;
pub(super) type ParseWith = MetaExpr<kw::parse_with>;
pub(super) type PreAssert = AssertLike<kw::pre_assert>;
pub(super) type Repr = MetaType<kw::repr>;
pub(super) type RestorePosition = MetaVoid<kw::restore_position>;
pub(super) type ReturnAllErrors = MetaVoid<kw::return_all_errors>;
pub(super) type ReturnUnexpectedError = MetaVoid<kw::return_unexpected_error>;
pub(super) type SeekBefore = MetaExpr<kw::seek_before>;
pub(super) type Stream = MetaIdent<kw::stream>;
pub(super) type Temp = MetaVoid<kw::temp>;
pub(super) type Try = MetaVoid<Token![try]>;
pub(super) type TryCalc = MetaExpr<kw::try_calc>;
pub(super) type TryMap = MetaExpr<kw::try_map>;
pub(super) type WriteWith = MetaExpr<kw::write_with>;
