use super::SpannedValue;
use crate::parser::{attrs, KeywordToken};
use core::convert::TryFrom;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Lit;

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum Kind {
    ByteStr(String),
    Char,
    Numeric(String),
}

impl core::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Kind::Char => "char",
                Kind::ByteStr(ty) | Kind::Numeric(ty) => ty,
            }
        )
    }
}

pub(crate) type Magic = Option<SpannedValue<Inner>>;

#[derive(Clone, Debug)]
pub(crate) struct Inner(Kind, TokenStream);

impl Inner {
    pub(crate) fn add_ref(&self) -> TokenStream {
        match &self.0 {
            Kind::ByteStr(_) => quote! { & },
            _ => TokenStream::new(),
        }
    }

    pub(crate) fn deref_value(&self) -> TokenStream {
        match self.0 {
            Kind::ByteStr(_) => {
                let value = &self.1;
                quote! { *#value }
            }
            _ => self.1.clone(),
        }
    }

    pub(crate) fn kind(&self) -> &Kind {
        &self.0
    }

    pub(crate) fn match_value(&self) -> &TokenStream {
        &self.1
    }
}

impl TryFrom<attrs::Magic> for SpannedValue<Inner> {
    type Error = syn::Error;

    fn try_from(magic: attrs::Magic) -> Result<Self, Self::Error> {
        let value = &magic.value;

        let kind = match &value {
            Lit::ByteStr(bytes) => Kind::ByteStr(format!("[u8; {}]", bytes.value().len())),
            Lit::Byte(_) => Kind::Numeric("u8".to_owned()),
            Lit::Char(_) => Kind::Char,
            Lit::Int(i) => Kind::Numeric(i.suffix().to_owned()),
            Lit::Float(f) => Kind::Numeric(f.suffix().to_owned()),
            Lit::Str(_) | Lit::Bool(_) | Lit::Verbatim(_) => {
                return Err(syn::Error::new(
                    value.span(),
                    "expected byte string, byte, char, float, or int",
                ))
            }
        };

        Ok(Self::new(
            Inner(kind, value.to_token_stream()),
            magic.keyword_span(),
        ))
    }
}
