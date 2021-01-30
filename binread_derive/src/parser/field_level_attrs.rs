use crate::binread_endian::Endian;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use super::{Assert, FromAttrs, PassedArgs, convert_assert, keywords as kw, meta_types::{MetaExpr, MetaFunc, MetaList, MetaLit}};
use syn::{Expr, Token, spanned::Spanned};

#[derive(Clone, Debug)]
pub(crate) enum Map {
    None,
    Map(TokenStream),
    Try(TokenStream),
}

impl Default for Map {
    fn default() -> Self {
        Self::None
    }
}

impl Map {
    pub fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum CondEndian {
    Fixed(Endian),
    Cond(Endian, TokenStream),
}

impl Default for CondEndian {
    fn default() -> Self {
        Self::Fixed(Endian::default())
    }
}

parse_any! {
    enum FieldLevelAttr {
        Big(kw::big),
        Little(kw::little),
        Default(kw::default),
        Ignore(kw::ignore),
        DerefNow(kw::deref_now),
        RestorePosition(kw::restore_position),
        PostProcessNow(kw::postprocess_now),
        Try(Token![try]),
        Temp(kw::temp),
        Map(MetaFunc<kw::map>),
        TryMap(MetaFunc<kw::try_map>),
        ParseWith(MetaFunc<kw::parse_with>),
        Magic(MetaLit<kw::magic>),
        Args(MetaList<kw::args, Expr>),
        ArgsTuple(MetaExpr<kw::args_tuple>),
        Assert(MetaList<kw::assert, Expr>),
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

#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct FieldLevelAttrs {
    pub args: PassedArgs,
    pub map: Map,
    pub ignore: bool,
    pub default: bool,
    pub calc: Option<TokenStream>,
    pub count: Option<TokenStream>,
    pub offset: Option<TokenStream>,
    pub offset_after: Option<TokenStream>,
    pub if_cond: Option<TokenStream>,
    pub deref_now: bool,
    pub postprocess_now: bool,
    pub restore_position: bool,
    pub do_try: bool,
    pub temp: bool,
    pub endian: CondEndian,
    pub assert: Vec<Assert>,
    pub magic: Option<TokenStream>,
    pub pad_before: Option<TokenStream>,
    pub pad_after: Option<TokenStream>,
    pub align_before: Option<TokenStream>,
    pub align_after: Option<TokenStream>,
    pub seek_before: Option<TokenStream>,
    pub pad_size_to: Option<TokenStream>,
    pub parse_with: Option<TokenStream>,
}

impl FromAttrs<FieldLevelAttr> for FieldLevelAttrs {
    #[allow(clippy::too_many_lines)]
    fn try_set_attr(&mut self, attr: FieldLevelAttr) -> syn::Result<()> {
        macro_rules! set_option {
            ($obj:ident.$field:ident, $raw_obj:ident) => { {
                only_first!($obj.$field, $raw_obj.ident);
                $obj.$field = Some($raw_obj.value.to_token_stream());
                Ok(())
            } }
        }

        macro_rules! set_bool {
            ($obj:ident.$field:ident, $kw:expr) => {
                if $obj.$field == false {
                    $obj.$field = true;
                    Ok(())
                } else {
                    super::duplicate_attr(&$kw)
                }
            }
        }

        fn set_endian(fla: &mut FieldLevelAttrs, endian: CondEndian, span: Span) -> syn::Result<()> {
            if matches!(fla.endian, CondEndian::Fixed(Endian::Native)) {
                fla.endian = endian;
                Ok(())
            } else {
                Err(syn::Error::new(span, "conflicting endian attribute"))
            }
        }

        fn set_map(fla: &mut FieldLevelAttrs, map: Map, span: Span) -> syn::Result<()> {
            if matches!(fla.map, Map::None) {
                fla.map = map;
                Ok(())
            } else {
                Err(syn::Error::new(span, "conflicting map-like attribute"))
            }
        }

        fn set_args(fla: &mut FieldLevelAttrs, args: PassedArgs, span: Span) -> syn::Result<()> {
            if matches!(fla.args, PassedArgs::None) {
                fla.args = args;
                Ok(())
            } else {
                Err(syn::Error::new(span, "conflicting args-like attribute"))
            }
        }

        match attr {
            FieldLevelAttr::Big(kw) =>
                set_endian(self, CondEndian::Fixed(Endian::Big), kw.span()),
            FieldLevelAttr::Little(kw) =>
                set_endian(self, CondEndian::Fixed(Endian::Little), kw.span()),
            FieldLevelAttr::Default(kw) =>
                set_bool!(self.default, kw),
            FieldLevelAttr::Ignore(kw) =>
                set_bool!(self.ignore, kw),
            FieldLevelAttr::DerefNow(kw) =>
                set_bool!(self.deref_now, kw),
            FieldLevelAttr::RestorePosition(kw) =>
                set_bool!(self.restore_position, kw),
            FieldLevelAttr::PostProcessNow(kw) =>
                set_bool!(self.postprocess_now, kw),
            FieldLevelAttr::Try(kw) =>
                set_bool!(self.do_try, kw),
            FieldLevelAttr::Temp(kw) =>
                set_bool!(self.temp, kw),
            FieldLevelAttr::Map(map) =>
                set_map(self, Map::Map(map.value.to_token_stream()), map.ident.span()),
            FieldLevelAttr::TryMap(map) =>
                set_map(self, Map::Try(map.value.to_token_stream()), map.ident.span()),
            FieldLevelAttr::ParseWith(parser) =>
                set_option!(self.parse_with, parser),
            FieldLevelAttr::Magic(magic) =>
                set_option!(self.magic, magic),
            FieldLevelAttr::Args(args) =>
                set_args(self, PassedArgs::List(args.get()), args.ident.span()),
            FieldLevelAttr::ArgsTuple(args) =>
                set_args(self, PassedArgs::Tuple(args.value.to_token_stream()), args.span()),
            FieldLevelAttr::Assert(assert) => {
                self.assert.push(convert_assert(&assert)?);
                Ok(())
            },
            FieldLevelAttr::Calc(calc) =>
                set_option!(self.calc, calc),
            FieldLevelAttr::Count(count) =>
                set_option!(self.count, count),
            FieldLevelAttr::IsLittle(is_little) =>
                set_endian(self, CondEndian::Cond(Endian::Little, is_little.to_token_stream()), is_little.ident.span()),
            FieldLevelAttr::IsBig(is_big) =>
                set_endian(self, CondEndian::Cond(Endian::Big, is_big.to_token_stream()), is_big.ident.span()),
            FieldLevelAttr::Offset(offset) =>
                set_option!(self.offset, offset),
            FieldLevelAttr::OffsetAfter(offset_after) =>
                set_option!(self.offset_after, offset_after),
            FieldLevelAttr::If(if_cond) =>
                set_option!(self.if_cond, if_cond),
            FieldLevelAttr::PadBefore(pad_before) =>
                set_option!(self.pad_before, pad_before),
            FieldLevelAttr::PadAfter(pad_after) =>
                set_option!(self.pad_after, pad_after),
            FieldLevelAttr::AlignBefore(align_before) =>
                set_option!(self.align_before, align_before),
            FieldLevelAttr::AlignAfter(align_after) =>
                set_option!(self.align_after, align_after),
            FieldLevelAttr::SeekBefore(seek_before) =>
                set_option!(self.seek_before, seek_before),
            FieldLevelAttr::PadSizeTo(pad_size_to) =>
                set_option!(self.pad_size_to, pad_size_to),
        }
    }
}
