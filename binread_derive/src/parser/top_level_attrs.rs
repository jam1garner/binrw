use crate::binread_endian::Endian;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use super::{Assert, collect_attrs, convert_assert, Imports, keywords as kw, MagicType, meta_types::{ImportArgTuple, IdentPatType, MetaFunc, MetaList, MetaLit, MetaType}};
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
        Repr(Box<MetaType<kw::repr>>),
        Import(MetaList<kw::import, IdentPatType>),
        ImportTuple(Box<ImportArgTuple>),
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

impl TopLevelAttrs {
    pub fn try_from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        fn set_endian(tla: &mut TopLevelAttrs, endian: Endian, span: Span) -> syn::Result<()> {
            if tla.endian == Endian::Native {
                tla.endian = endian;
                Ok(())
            } else {
                Err(syn::Error::new(span, "Conflicting endian keywords"))
            }
        }

        fn set_error(tla: &mut TopLevelAttrs, error: EnumErrorHandling, span: Span) -> syn::Result<()> {
            if tla.return_error_mode == EnumErrorHandling::Default {
                tla.return_error_mode = error;
                Ok(())
            } else {
                Err(syn::Error::new(span, "Conflicting error mode keywords"))
            }
        }

        let mut tla = Self::default();
        let attrs = collect_attrs::<TopLevelAttr>(attrs)?;

        for attr in attrs {
            match attr {
                TopLevelAttr::Big(kw) => {
                    set_endian(&mut tla, Endian::Big, kw.span())?;
                },
                TopLevelAttr::Little(kw) => {
                    set_endian(&mut tla, Endian::Little, kw.span())?;
                },
                TopLevelAttr::Import(s) => {
                    only_first!(tla.import, s.ident.span());
                    let (idents, tys): (Vec<_>, Vec<_>) = s.fields
                        .iter()
                        .cloned()
                        .map(|import_arg| (import_arg.ident, import_arg.ty))
                        .unzip();
                    tla.import = Imports::List(idents, tys);
                },
                TopLevelAttr::ImportTuple(s) => {
                    only_first!(tla.import, s.ident.span());
                    tla.import = Imports::Tuple(s.arg.ident.clone(), s.arg.ty.clone().into());
                },
                TopLevelAttr::Assert(a) => {
                    tla.assert.push(convert_assert(&a)?);
                },
                TopLevelAttr::PreAssert(a) => {
                    tla.pre_assert.push(convert_assert(&a)?);
                },
                TopLevelAttr::Repr(ty) => {
                    only_first!(tla.repr, ty.ident.span());
                    tla.repr = Some(ty.value);
                },
                TopLevelAttr::ReturnAllErrors(e) => {
                    set_error(&mut tla, EnumErrorHandling::ReturnAllErrors, e.span())?;
                },
                TopLevelAttr::ReturnUnexpectedError(e) => {
                    set_error(&mut tla, EnumErrorHandling::ReturnUnexpectedError, e.span())?;
                },
                TopLevelAttr::Magic(m) => {
                    only_first!(tla.magic, m.ident.span());
                    tla.magic = Some((magic_to_type(&m), magic_to_tokens(&m)));
                },
                TopLevelAttr::Map(m) => {
                    only_first!(tla.map, m.ident.span());
                    tla.map = Some(m.into_token_stream());
                }
            }
        }
        Ok(tla)
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
