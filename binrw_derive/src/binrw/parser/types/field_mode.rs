use crate::{
    binrw::parser::{attrs, TrySet},
    meta_types::KeywordToken,
};
use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Clone, Debug)]
pub(crate) enum FieldMode {
    Normal,
    Default,
    Calc(TokenStream),
    TryCalc(TokenStream),
    Function(TokenStream),
}

impl Default for FieldMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<attrs::Ignore> for FieldMode {
    fn from(_: attrs::Ignore) -> Self {
        Self::Default
    }
}

impl From<attrs::Default> for FieldMode {
    fn from(_: attrs::Default) -> Self {
        Self::Default
    }
}

impl From<attrs::Calc> for FieldMode {
    fn from(calc: attrs::Calc) -> Self {
        Self::Calc(calc.into_token_stream())
    }
}

impl From<attrs::TryCalc> for FieldMode {
    fn from(calc: attrs::TryCalc) -> Self {
        Self::TryCalc(calc.into_token_stream())
    }
}

impl From<attrs::ParseWith> for FieldMode {
    fn from(parse_with: attrs::ParseWith) -> Self {
        Self::Function(parse_with.into_token_stream())
    }
}

impl From<attrs::WriteWith> for FieldMode {
    fn from(write_with: attrs::WriteWith) -> Self {
        Self::Function(write_with.into_token_stream())
    }
}

impl<T: Into<FieldMode> + KeywordToken> TrySet<FieldMode> for T {
    fn try_set(self, to: &mut FieldMode) -> syn::Result<()> {
        if matches!(*to, FieldMode::Normal) {
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
