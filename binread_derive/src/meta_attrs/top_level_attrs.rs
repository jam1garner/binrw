use super::*;
use super::parser::{TopLevelAttr, MetaAttrList, MetaLit};
use proc_macro2::Span;
use quote::ToTokens;
use crate::binread_endian::Endian;

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

#[derive(Debug, Default, Clone)]
pub struct TopLevelAttrs {
    // ======================
    //  Top-Only Attributes
    // ======================
    pub import: Imports,
    pub repr: Option<Type>,
    pub return_error_mode: EnumErrorHandling,

    // ======================
    //  All-level attributes
    // ======================
    pub endian: Endian,

    // assertions/error handling
    pub assert: Vec<Assert>,
    pub magic: Option<TokenStream>,
    pub magic_type: Option<MagicType>,
    pub pre_assert: Vec<Assert>,

    // other
    pub map: Option<TokenStream>,
}

impl TopLevelAttrs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        fn set_endian(tla: &mut TopLevelAttrs, endian: Endian, span: &Span) -> syn::Result<()> {
            if tla.endian == Endian::Native {
                tla.endian = endian;
                Ok(())
            } else {
                Err(syn::Error::new(*span, "Conflicting endianness keywords"))
            }
        }

        fn set_error(tla: &mut TopLevelAttrs, error: EnumErrorHandling, span: &Span) -> syn::Result<()> {
            if tla.return_error_mode == EnumErrorHandling::Default {
                tla.return_error_mode = error;
                Ok(())
            } else {
                Err(syn::Error::new(*span, "Conflicting error mode"))
            }
        }

        let mut tla = Self::new();
        let attrs = collect_attrs::<TopLevelAttr>(attrs)?;

        for attr in attrs {
            match attr {
                TopLevelAttr::Big(kw) => {
                    set_endian(&mut tla, Endian::Big, &kw.span)?;
                },
                TopLevelAttr::Little(kw) => {
                    set_endian(&mut tla, Endian::Little, &kw.span)?;
                },
                TopLevelAttr::Import(s) => {
                    if tla.import.is_some() {
                        return Err(syn::Error::new(s.ident.span, "Conflicting import"));
                    }

                    let (idents, tys): (Vec<_>, Vec<_>) = s.fields
                        .iter()
                        .cloned()
                        .map(|import_arg| (import_arg.ident, import_arg.ty))
                        .unzip();
                    tla.import = Imports::List(idents, tys);
                },
                TopLevelAttr::ImportTuple(s) => {
                    if tla.import.is_some() {
                        return Err(syn::Error::new(s.ident.span, "Conflicting import"));
                    }

                    tla.import = Imports::Tuple(s.arg.ident.clone(), s.arg.ty.clone().into());
                },
                TopLevelAttr::Assert(a) => {
                    tla.assert.push(convert_assert(&a)?);
                },
                TopLevelAttr::PreAssert(a) => {
                    tla.pre_assert.push(convert_assert(&a)?);
                },
                TopLevelAttr::Repr(ty) => {
                    if tla.repr.is_some() {
                        return Err(syn::Error::new(ty.ident.span, "Conflicting repr keywords"))
                    }
                    tla.repr = Some(ty.value);
                },
                TopLevelAttr::ReturnAllErrors(e) => {
                    set_error(&mut tla, EnumErrorHandling::ReturnAllErrors, &e.span)?;
                },
                TopLevelAttr::ReturnUnexpectedError(e) => {
                    set_error(&mut tla, EnumErrorHandling::ReturnUnexpectedError, &e.span)?;
                },
                TopLevelAttr::Magic(m) => {
                    if tla.magic.is_some() {
                        return Err(syn::Error::new(m.ident.span, "Conflicting magic"));
                    }
                    tla.magic = Some(magic_to_tokens(&m));
                    tla.magic_type = Some(magic_to_type(&m));
                },
                TopLevelAttr::Map(m) => {
                    if tla.map.is_some() {
                        return Err(syn::Error::new(m.ident.span, "Conflicting map"));
                    }
                    tla.map = Some(m.into_token_stream());
                }
            }
        }
        Ok(tla)
    }
}

fn magic_to_type(magic: &MetaLit<impl syn::parse::Parse>) -> MagicType {
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

fn magic_to_tokens(magic: &MetaLit<impl syn::parse::Parse>) -> TokenStream {
    let magic = &magic.value;
    if let Lit::Str(_) | Lit::ByteStr(_) = magic {
        quote::quote!{
            *#magic
        }
    } else {
        magic.to_token_stream()
    }
}
