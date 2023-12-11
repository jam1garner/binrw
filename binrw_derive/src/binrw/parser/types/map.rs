use crate::{
    binrw::parser::{attrs, TrySet},
    meta_types::KeywordToken,
};
use proc_macro2::TokenStream;
use quote::ToTokens;

// Lint: Makes code less clear
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub(crate) enum Map {
    None,
    Map(TokenStream),
    Try(TokenStream),
    Repr(TokenStream),
}

impl Map {
    pub(crate) fn as_repr(&self) -> Option<&TokenStream> {
        match self {
            Map::Repr(r) => Some(r),
            _ => None,
        }
    }

    pub(crate) fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub(crate) fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub(crate) fn is_try(&self) -> bool {
        matches!(self, Self::Try(_) | Self::Repr(_))
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::None
    }
}

impl From<attrs::Map> for Map {
    fn from(map: attrs::Map) -> Self {
        Self::Map(map.value.to_token_stream())
    }
}

impl From<attrs::TryMap> for Map {
    fn from(try_map: attrs::TryMap) -> Self {
        Self::Try(try_map.value.to_token_stream())
    }
}

impl From<attrs::Repr> for Map {
    fn from(repr: attrs::Repr) -> Self {
        Self::Repr(repr.value.to_token_stream())
    }
}

impl<T: Into<Map> + KeywordToken> TrySet<Map> for T {
    fn try_set(self, to: &mut Map) -> syn::Result<()> {
        if to.is_some() {
            Err(syn::Error::new(
                self.keyword_span(),
                "conflicting map keyword",
            ))
        } else {
            *to = self.into();
            Ok(())
        }
    }
}
