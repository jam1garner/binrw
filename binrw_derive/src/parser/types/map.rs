use crate::parser::{attrs, KeywordToken, TrySet};
use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Clone, Debug)]
pub(crate) enum Map {
    None,
    Map(TokenStream),
    Try(TokenStream),
}

impl Map {
    pub(crate) fn is_some(&self) -> bool {
        !matches!(self, Self::None)
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
