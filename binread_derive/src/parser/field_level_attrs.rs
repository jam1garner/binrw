use crate::binread_endian::Endian;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use super::{Assert, Check, FromAttrs, KeywordToken, PassedArgs, convert_assert, keywords as kw, meta_types::{MetaExpr, MetaList, MetaLit}, set_option_ts};
use syn::{Expr, Field, Token, spanned::Spanned};

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
        Map(MetaExpr<kw::map>),
        TryMap(MetaExpr<kw::try_map>),
        ParseWith(MetaExpr<kw::parse_with>),
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

struct FieldCheck;
impl Check<FieldLevelAttr> for FieldCheck {
    const ERR_LOCATION: &'static str = "field";
    fn check(_: &FieldLevelAttr) -> syn::Result<()> {
        Ok(())
    }
}

impl FieldLevelAttrs {
    pub(crate) fn try_from_field(field: &Field) -> syn::Result<Self> {
        Self::try_from_attrs::<FieldCheck>(&field.attrs)
    }

    fn set_endian(&mut self, endian: CondEndian, span: Span) -> syn::Result<()> {
        if matches!(self.endian, CondEndian::Fixed(Endian::Native)) {
            self.endian = endian;
            Ok(())
        } else {
            Err(syn::Error::new(span, "conflicting endian attribute"))
        }
    }

    fn set_map(&mut self, map: Map, span: Span) -> syn::Result<()> {
        if matches!(self.map, Map::None) {
            self.map = map;
            Ok(())
        } else {
            Err(syn::Error::new(span, "conflicting map-like attribute"))
        }
    }

    fn set_args(&mut self, args: PassedArgs, span: Span) -> syn::Result<()> {
        if matches!(self.args, PassedArgs::None) {
            self.args = args;
            Ok(())
        } else {
            Err(syn::Error::new(span, "conflicting args-like attribute"))
        }
    }
}

impl FromAttrs<FieldLevelAttr> for FieldLevelAttrs {
    fn try_set_attr<C: Check<FieldLevelAttr>>(&mut self, attr: FieldLevelAttr) -> syn::Result<()> {
        C::check(&attr)?;
        match attr {
            FieldLevelAttr::Big(kw) =>
                self.set_endian(CondEndian::Fixed(Endian::Big), kw.span()),
            FieldLevelAttr::Little(kw) =>
                self.set_endian(CondEndian::Fixed(Endian::Little), kw.span()),
            FieldLevelAttr::Default(kw) =>
                set_bool(&mut self.default, &kw),
            FieldLevelAttr::Ignore(kw) =>
                set_bool(&mut self.ignore, &kw),
            FieldLevelAttr::DerefNow(kw) =>
                set_bool(&mut self.deref_now, &kw),
            FieldLevelAttr::RestorePosition(kw) =>
                set_bool(&mut self.restore_position, &kw),
            FieldLevelAttr::PostProcessNow(kw) =>
                set_bool(&mut self.postprocess_now, &kw),
            FieldLevelAttr::Try(kw) =>
                set_bool(&mut self.do_try, &kw),
            FieldLevelAttr::Temp(kw) =>
                set_bool(&mut self.temp, &kw),
            FieldLevelAttr::Map(map) =>
                self.set_map(Map::Map(map.value.to_token_stream()), map.ident.span()),
            FieldLevelAttr::TryMap(map) =>
                self.set_map(Map::Try(map.value.to_token_stream()), map.ident.span()),
            FieldLevelAttr::ParseWith(parser) =>
                set_option_ts(&mut self.parse_with, &parser),
            FieldLevelAttr::Magic(magic) =>
                set_option_ts(&mut self.magic, &magic),
            FieldLevelAttr::Args(args) =>
                self.set_args(PassedArgs::List(args.get()), args.ident.span()),
            FieldLevelAttr::ArgsTuple(args) =>
                self.set_args(PassedArgs::Tuple(args.value.to_token_stream()), args.span()),
            FieldLevelAttr::Assert(assert) => {
                self.assert.push(convert_assert(&assert)?);
                Ok(())
            },
            FieldLevelAttr::Calc(calc) =>
                set_option_ts(&mut self.calc, &calc),
            FieldLevelAttr::Count(count) =>
                set_option_ts(&mut self.count, &count),
            FieldLevelAttr::IsLittle(is_little) =>
                self.set_endian(CondEndian::Cond(Endian::Little, is_little.to_token_stream()), is_little.ident.span()),
            FieldLevelAttr::IsBig(is_big) =>
                self.set_endian(CondEndian::Cond(Endian::Big, is_big.to_token_stream()), is_big.ident.span()),
            FieldLevelAttr::Offset(offset) =>
                set_option_ts(&mut self.offset, &offset),
            FieldLevelAttr::OffsetAfter(offset_after) =>
                set_option_ts(&mut self.offset_after, &offset_after),
            FieldLevelAttr::If(if_cond) =>
                set_option_ts(&mut self.if_cond, &if_cond),
            FieldLevelAttr::PadBefore(pad_before) =>
                set_option_ts(&mut self.pad_before, &pad_before),
            FieldLevelAttr::PadAfter(pad_after) =>
                set_option_ts(&mut self.pad_after, &pad_after),
            FieldLevelAttr::AlignBefore(align_before) =>
                set_option_ts(&mut self.align_before, &align_before),
            FieldLevelAttr::AlignAfter(align_after) =>
                set_option_ts(&mut self.align_after, &align_after),
            FieldLevelAttr::SeekBefore(seek_before) =>
                set_option_ts(&mut self.seek_before, &seek_before),
            FieldLevelAttr::PadSizeTo(pad_size_to) =>
                set_option_ts(&mut self.pad_size_to, &pad_size_to),
        }
    }
}

fn set_bool<Keyword: KeywordToken + Spanned>(value: &mut bool, kw: &Keyword) -> syn::Result<()> {
    if *value {
        super::duplicate_attr(kw)
    } else {
        *value = true;
        Ok(())
    }
}
