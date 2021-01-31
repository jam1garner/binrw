use crate::binread_endian::Endian;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use super::{Assert, Check, FromAttrs, Imports, KeywordToken, MagicType, convert_assert, keywords as kw, meta_types::{ImportArgTuple, IdentPatType, MetaFunc, MetaList, MetaLit, MetaType}, set_option_ts};
use syn::{DeriveInput, Expr, Lit, Variant, spanned::Spanned};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum EnumErrorHandling {
    Default,
    ReturnAllErrors,
    ReturnUnexpectedError,
}

impl Default for EnumErrorHandling {
    fn default() -> Self {
        Self::Default
    }
}

parse_any! {
    enum TopLevelAttr {
        Big(kw::big),
        Little(kw::little),
        ReturnAllErrors(kw::return_all_errors),
        ReturnUnexpectedError(kw::return_unexpected_error),
        Magic(MetaLit<kw::magic>),
        Repr(MetaType<kw::repr>),
        Import(MetaList<kw::import, IdentPatType>),
        ImportTuple(ImportArgTuple),
        Assert(MetaList<kw::assert, Expr>),
        PreAssert(MetaList<kw::pre_assert, Expr>),
        Map(MetaFunc<kw::map>),
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct TopLevelAttrs {
    pub endian: Endian,
    pub import: Imports,
    pub assert: Vec<Assert>,
    pub pre_assert: Vec<Assert>,
    pub magic: Option<(MagicType, TokenStream)>,
    pub map: Option<TokenStream>,
    pub repr: Option<TokenStream>,
    pub return_error_mode: EnumErrorHandling,
}

pub(crate) struct StructCheck;
impl Check<TopLevelAttr> for StructCheck {
    const ERR_LOCATION: &'static str = "struct";
    fn check(attr: &TopLevelAttr) -> syn::Result<()> {
        match attr {
            TopLevelAttr::Big(_)
            | TopLevelAttr::Little(_)
            | TopLevelAttr::Magic(_)
            | TopLevelAttr::Import(_)
            | TopLevelAttr::ImportTuple(_)
            | TopLevelAttr::Assert(_)
            | TopLevelAttr::PreAssert(_)
            | TopLevelAttr::Map(_) => Ok(()),
            TopLevelAttr::ReturnAllErrors(kw) => Self::err(kw),
            TopLevelAttr::ReturnUnexpectedError(kw) => Self::err(kw),
            TopLevelAttr::Repr(repr) => Self::err(&repr.ident)
        }
    }
}

pub(crate) struct UnitEnumCheck;
impl Check<TopLevelAttr> for UnitEnumCheck {
    const ERR_LOCATION: &'static str = "unit enum";
    fn check(attr: &TopLevelAttr) -> syn::Result<()> {
        match attr {
            TopLevelAttr::Big(_)
            | TopLevelAttr::Little(_)
            | TopLevelAttr::Repr(_) => Ok(()),
            TopLevelAttr::Magic(m) => Self::err(&m.ident),
            TopLevelAttr::Import(i) => Self::err(&i.ident),
            TopLevelAttr::ImportTuple(i) => Self::err(&i.ident),
            TopLevelAttr::Assert(a) => Self::err(&a.ident),
            TopLevelAttr::PreAssert(a) => Self::err(&a.ident),
            TopLevelAttr::Map(m) => Self::err(&m.ident),
            TopLevelAttr::ReturnAllErrors(kw) => Self::err(kw),
            TopLevelAttr::ReturnUnexpectedError(kw) => Self::err(kw),
        }
    }
}

pub(crate) struct VariantEnumCheck;
impl Check<TopLevelAttr> for VariantEnumCheck {
    const ERR_LOCATION: &'static str = "enum";
    fn check(attr: &TopLevelAttr) -> syn::Result<()> {
        match attr {
            TopLevelAttr::Big(_)
            | TopLevelAttr::Little(_)
            | TopLevelAttr::Magic(_)
            | TopLevelAttr::Import(_)
            | TopLevelAttr::ImportTuple(_)
            | TopLevelAttr::Assert(_)
            | TopLevelAttr::PreAssert(_)
            | TopLevelAttr::Map(_)
            | TopLevelAttr::ReturnAllErrors(_)
            | TopLevelAttr::ReturnUnexpectedError(_) => Ok(()),
            TopLevelAttr::Repr(repr) => Self::err(&repr.ident),
        }
    }
}

impl TopLevelAttrs {
    pub(crate) fn try_from_input(input: &DeriveInput) -> syn::Result<Self> {
        match &input.data {
            syn::Data::Struct(_) =>
                Self::try_from_attrs::<StructCheck>(&input.attrs),
            syn::Data::Enum(en) => {
                if en.variants.iter().all(crate::codegen::read_options::no_variant_data) {
                    Self::try_from_attrs::<UnitEnumCheck>(&input.attrs)
                } else {
                    Self::try_from_attrs::<VariantEnumCheck>(&input.attrs)
                }
            },
            syn::Data::Union(union) =>
                Err(syn::Error::new(union.union_token.span, "Unions are not supported"))
        }
    }

    pub(crate) fn try_from_variant(variant: &Variant) -> syn::Result<Self> {
        Self::try_from_attrs::<VariantEnumCheck>(&variant.attrs)
    }

    fn set_endian(&mut self, endian: Endian, span: Span) -> syn::Result<()> {
        if self.endian == Endian::Native {
            self.endian = endian;
            Ok(())
        } else {
            Err(syn::Error::new(span, "conflicting endian attribute"))
        }
    }

    fn set_error(&mut self, error: EnumErrorHandling, span: Span) -> syn::Result<()> {
        if self.return_error_mode == EnumErrorHandling::Default {
            self.return_error_mode = error;
            Ok(())
        } else {
            Err(syn::Error::new(span, "conflicting error mode attribute"))
        }
    }

    fn set_import<K: KeywordToken + Spanned, F: Fn() -> Imports>(&mut self, get_import: F, kw: &K) -> syn::Result<()> {
        if self.import.is_some() {
            super::duplicate_attr(kw)
        } else {
            self.import = get_import();
            Ok(())
        }
    }
}

impl FromAttrs<TopLevelAttr> for TopLevelAttrs {
    fn try_set_attr<C: Check<TopLevelAttr>>(&mut self, attr: TopLevelAttr) -> syn::Result<()> {
        C::check(&attr)?;
        match attr {
            TopLevelAttr::Big(kw) =>
                self.set_endian(Endian::Big, kw.span())?,
            TopLevelAttr::Little(kw) =>
                self.set_endian(Endian::Little, kw.span())?,
            TopLevelAttr::Import(s) =>
                self.set_import(|| {
                    let (idents, tys): (Vec<_>, Vec<_>) = s.fields
                        .iter()
                        .cloned()
                        .map(|import_arg| (import_arg.ident, import_arg.ty))
                        .unzip();
                    Imports::List(idents, tys)
                }, &s.ident)?,
            TopLevelAttr::ImportTuple(s) =>
                self.set_import(|| Imports::Tuple(s.arg.ident.clone(), s.arg.ty.clone().into()), &s.ident)?,
            TopLevelAttr::Assert(a) =>
                self.assert.push(convert_assert(&a)?),
            TopLevelAttr::PreAssert(a) =>
                self.pre_assert.push(convert_assert(&a)?),
            TopLevelAttr::Repr(ty) =>
                set_option_ts(&mut self.repr, &ty)?,
            TopLevelAttr::ReturnAllErrors(e) =>
                self.set_error(EnumErrorHandling::ReturnAllErrors, e.span())?,
            TopLevelAttr::ReturnUnexpectedError(e) =>
                self.set_error(EnumErrorHandling::ReturnUnexpectedError, e.span())?,
            TopLevelAttr::Magic(m) =>
                if self.magic.is_some() {
                    return super::duplicate_attr(&m.ident);
                } else {
                    self.magic = Some((magic_to_type(&m), magic_to_tokens(&m)));
                },
            TopLevelAttr::Map(m) =>
                set_option_ts(&mut self.map, &m)?,
        }

        Ok(())
    }
}

fn magic_to_type<Keyword>(magic: &MetaLit<Keyword>) -> MagicType {
    let magic = &magic.value;
    match magic {
        Lit::Str(_) => MagicType::Str,
        Lit::ByteStr(_) => MagicType::ByteStr,
        Lit::Byte(_) => MagicType::Byte,
        Lit::Char(_) => MagicType::Char,
        Lit::Int(i) => MagicType::Int(i.suffix().to_owned()),
        Lit::Float(_) => MagicType::Float,
        Lit::Bool(_) => MagicType::Bool,
        Lit::Verbatim(_) => MagicType::Verbatim
    }
}

fn magic_to_tokens<Keyword>(magic: &MetaLit<Keyword>) -> TokenStream {
    let magic = &magic.value;
    if let Lit::Str(_) | Lit::ByteStr(_) = magic {
        quote::quote!{
            *#magic
        }
    } else {
        magic.to_token_stream()
    }
}
