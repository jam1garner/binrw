use crate::parser::{attrs, KeywordToken, TrySet};
use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Clone, Debug)]
pub(crate) enum ReadMode {
    Normal,
    Default,
    Calc(TokenStream),
    ParseWith(TokenStream),
}

impl Default for ReadMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<attrs::Ignore> for ReadMode {
    fn from(_: attrs::Ignore) -> Self {
        Self::Default
    }
}

impl From<attrs::Default> for ReadMode {
    fn from(_: attrs::Default) -> Self {
        Self::Default
    }
}

impl From<attrs::Calc> for ReadMode {
    fn from(calc: attrs::Calc) -> Self {
        Self::Calc(calc.into_token_stream())
    }
}

impl From<attrs::ParseWith> for ReadMode {
    fn from(parse_with: attrs::ParseWith) -> Self {
        Self::ParseWith(parse_with.into_token_stream())
    }
}

impl<T: Into<ReadMode> + KeywordToken> TrySet<ReadMode> for T {
    fn try_set(self, to: &mut ReadMode) -> syn::Result<()> {
        if matches!(*to, ReadMode::Normal) {
            *to = self.into();
            Ok(())
        } else {
            Err(syn::Error::new(
                self.keyword_span(),
                "conflicting read mode keyword",
            ))
        }
    }
}
