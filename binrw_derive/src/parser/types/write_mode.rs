use crate::parser::{read::attrs, KeywordToken, TrySet};
use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Clone, Debug)]
pub(crate) enum WriteMode {
    Normal,
    Ignore,
    Calc(TokenStream),
    WriteWith(TokenStream),
}

impl Default for WriteMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<attrs::Ignore> for WriteMode {
    fn from(_: attrs::Ignore) -> Self {
        Self::Ignore
    }
}

impl From<attrs::Calc> for WriteMode {
    fn from(calc: attrs::Calc) -> Self {
        Self::Calc(calc.into_token_stream())
    }
}

impl From<attrs::WriteWith> for WriteMode {
    fn from(write_with: attrs::WriteWith) -> Self {
        Self::WriteWith(write_with.into_token_stream())
    }
}

impl<T: Into<WriteMode> + KeywordToken> TrySet<WriteMode> for T {
    fn try_set(self, to: &mut WriteMode) -> syn::Result<()> {
        if matches!(*to, WriteMode::Normal) {
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
