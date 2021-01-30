use crate::binread_endian::Endian;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use super::{Assert, FromAttrs, convert_assert, Imports, keywords as kw, MagicType, meta_types::{ImportArgTuple, IdentPatType, MetaFunc, MetaList, MetaLit, MetaType}};
use syn::{Expr, Lit, Type, spanned::Spanned};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EnumErrorHandling {
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
pub struct TopLevelAttrs {
    pub import: Imports,
    pub endian: Endian,
    pub assert: Vec<Assert>,
    pub pre_assert: Vec<Assert>,

    // TODO: Used for enum only
    pub repr: Option<Type>,

    // TODO: Used for variants only?
    pub return_error_mode: EnumErrorHandling,
    pub magic: Option<(MagicType, TokenStream)>,
    pub map: Option<TokenStream>,
}

impl FromAttrs<TopLevelAttr> for TopLevelAttrs {
    fn try_set_attr(&mut self, attr: TopLevelAttr) -> syn::Result<()> {
        fn set_endian(tla: &mut TopLevelAttrs, endian: Endian, span: Span) -> syn::Result<()> {
            if tla.endian == Endian::Native {
                tla.endian = endian;
                Ok(())
            } else {
                Err(syn::Error::new(span, "conflicting endian attribute"))
            }
        }

        fn set_error(tla: &mut TopLevelAttrs, error: EnumErrorHandling, span: Span) -> syn::Result<()> {
            if tla.return_error_mode == EnumErrorHandling::Default {
                tla.return_error_mode = error;
                Ok(())
            } else {
                Err(syn::Error::new(span, "conflicting error mode attribute"))
            }
        }

        match attr {
            TopLevelAttr::Big(kw) =>
                set_endian(self, Endian::Big, kw.span())?,
            TopLevelAttr::Little(kw) =>
                set_endian(self, Endian::Little, kw.span())?,
            TopLevelAttr::Import(s) => {
                only_first!(self.import, s.ident);
                let (idents, tys): (Vec<_>, Vec<_>) = s.fields
                    .iter()
                    .cloned()
                    .map(|import_arg| (import_arg.ident, import_arg.ty))
                    .unzip();
                self.import = Imports::List(idents, tys);
            },
            TopLevelAttr::ImportTuple(s) => {
                only_first!(self.import, s.ident);
                self.import = Imports::Tuple(s.arg.ident, s.arg.ty.into());
            },
            TopLevelAttr::Assert(a) =>
                self.assert.push(convert_assert(&a)?),
            TopLevelAttr::PreAssert(a) =>
                self.pre_assert.push(convert_assert(&a)?),
            TopLevelAttr::Repr(ty) => {
                only_first!(self.repr, ty.ident);
                self.repr = Some(ty.value);
            },
            TopLevelAttr::ReturnAllErrors(e) =>
                set_error(self, EnumErrorHandling::ReturnAllErrors, e.span())?,
            TopLevelAttr::ReturnUnexpectedError(e) =>
                set_error(self, EnumErrorHandling::ReturnUnexpectedError, e.span())?,
            TopLevelAttr::Magic(m) => {
                only_first!(self.magic, m.ident);
                self.magic = Some((magic_to_type(&m), magic_to_tokens(&m)));
            },
            TopLevelAttr::Map(m) => {
                only_first!(self.map, m.ident);
                self.map = Some(m.into_token_stream());
            }
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
