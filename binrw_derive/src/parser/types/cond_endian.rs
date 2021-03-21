use crate::parser::{attrs, KeywordToken, TrySet};
use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Endian {
    Big,
    Little,
}

#[derive(Clone, Debug)]
pub(crate) enum CondEndian {
    Inherited,
    Fixed(Endian),
    Cond(Endian, TokenStream),
}

impl CondEndian {
    pub(crate) fn is_some(&self) -> bool {
        !matches!(self, CondEndian::Inherited)
    }
}

impl Default for CondEndian {
    fn default() -> Self {
        Self::Inherited
    }
}

impl From<attrs::Big> for CondEndian {
    fn from(_: attrs::Big) -> Self {
        Self::Fixed(Endian::Big)
    }
}

impl From<attrs::Little> for CondEndian {
    fn from(_: attrs::Little) -> Self {
        Self::Fixed(Endian::Little)
    }
}

impl From<attrs::IsBig> for CondEndian {
    fn from(is_big: attrs::IsBig) -> Self {
        Self::Cond(Endian::Big, is_big.value.to_token_stream())
    }
}

impl From<attrs::IsLittle> for CondEndian {
    fn from(is_little: attrs::IsLittle) -> Self {
        Self::Cond(Endian::Little, is_little.value.to_token_stream())
    }
}

impl<T: Into<CondEndian> + KeywordToken> TrySet<CondEndian> for T {
    fn try_set(self, to: &mut CondEndian) -> syn::Result<()> {
        if matches!(*to, CondEndian::Inherited) {
            *to = self.into();
            Ok(())
        } else {
            Err(syn::Error::new(
                self.keyword_span(),
                "conflicting endianness keyword",
            ))
        }
    }
}
