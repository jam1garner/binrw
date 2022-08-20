use super::SpannedValue;
use crate::parser::{attrs, KeywordToken};
use core::{convert::TryFrom, fmt::Display};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Lit;

#[derive(PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
pub(crate) enum Kind {
    Numeric(String),
    ByteStr(String),
}

impl From<&Kind> for TokenStream {
    fn from(kind: &Kind) -> Self {
        match kind {
            Kind::ByteStr(ty) | Kind::Numeric(ty) => {
                let ty: TokenStream = ty.parse().unwrap();
                quote! { #ty }
            }
        }
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Kind::ByteStr(ty) | Kind::Numeric(ty) => Display::fmt(ty, f),
        }
    }
}

pub(crate) type Magic = Option<SpannedValue<Inner>>;

#[derive(Clone, Debug)]
pub(crate) struct Inner(Kind, TokenStream);

impl Inner {
    pub(crate) fn add_ref(&self) -> TokenStream {
        match &self.0 {
            Kind::ByteStr(_) => quote! { & },
            Kind::Numeric(_) => TokenStream::new(),
        }
    }

    pub(crate) fn deref_value(&self) -> TokenStream {
        match self.0 {
            Kind::ByteStr(_) => {
                let value = &self.1;
                quote! { *#value }
            }
            Kind::Numeric(_) => self.1.clone(),
        }
    }

    pub(crate) fn kind(&self) -> &Kind {
        &self.0
    }

    pub(crate) fn match_value(&self) -> &TokenStream {
        &self.1
    }

    #[cfg(all(nightly, not(coverage)))] // Not actually nightly, but only used by nightly mod syntax_highlighting
    pub(crate) fn into_match_value(self) -> TokenStream {
        self.1
    }
}

impl TryFrom<attrs::Magic> for SpannedValue<Inner> {
    type Error = syn::Error;

    fn try_from(magic: attrs::Magic) -> Result<Self, Self::Error> {
        let value = &magic.value;

        let kind = match &value {
            Lit::ByteStr(bytes) => Kind::ByteStr(format!("[u8; {}]", bytes.value().len())),
            Lit::Byte(_) => Kind::Numeric("u8".to_owned()),
            Lit::Int(i) => {
                if i.suffix().is_empty() {
                    return Err(syn::Error::new(
                        value.span(),
                        format!(
                            "expected explicit type suffix for integer literal\ne.g {}u64",
                            i
                        ),
                    ));
                }
                Kind::Numeric(i.suffix().to_owned())
            }
            Lit::Float(f) => {
                if f.suffix().is_empty() {
                    return Err(syn::Error::new(
                        value.span(),
                        format!(
                            "expected explicit type suffix for float literal\nvalid values are {0}f32 or {0}f64",
                            f
                        ),
                    ));
                }
                Kind::Numeric(f.suffix().to_owned())
            }
            Lit::Char(_) | Lit::Str(_) | Lit::Bool(_) | Lit::Verbatim(_) => {
                return Err(syn::Error::new(
                    value.span(),
                    "expected byte string, byte, float, or int",
                ))
            }
        };

        Ok(Self::new(
            Inner(kind, value.to_token_stream()),
            magic.keyword_span(),
        ))
    }
}
