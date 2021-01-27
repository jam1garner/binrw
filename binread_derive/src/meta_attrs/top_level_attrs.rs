use super::{Assert, Imports, MagicType, collect_attrs, convert_assert, parser::{TopLevelAttr, MetaLit}};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{Lit, Type, spanned::Spanned};
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
    pub magic: Option<(MagicType, TokenStream)>,
    pub pre_assert: Vec<Assert>,

    // other
    pub map: Option<TokenStream>,
}

impl TopLevelAttrs {
    pub fn try_from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        macro_rules! only_first {
            ($obj:ident.$field:ident, $span:expr) => {
                if $obj.$field.is_some() {
                    return Err(syn::Error::new($span, concat!("Conflicting ", stringify!($field), " keywords")));
                }
            }
        }

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
