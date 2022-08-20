use crate::parser::{attrs, KeywordToken, TrySet};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum EnumErrorMode {
    Default,
    ReturnAllErrors,
    ReturnUnexpectedError,
}

impl Default for EnumErrorMode {
    fn default() -> Self {
        Self::Default
    }
}

impl From<attrs::ReturnAllErrors> for EnumErrorMode {
    fn from(_: attrs::ReturnAllErrors) -> Self {
        Self::ReturnAllErrors
    }
}

impl From<attrs::ReturnUnexpectedError> for EnumErrorMode {
    fn from(_: attrs::ReturnUnexpectedError) -> Self {
        Self::ReturnUnexpectedError
    }
}

impl<T: Into<EnumErrorMode> + KeywordToken> TrySet<EnumErrorMode> for T {
    fn try_set(self, to: &mut EnumErrorMode) -> syn::Result<()> {
        if *to == EnumErrorMode::Default {
            *to = self.into();
            Ok(())
        } else {
            Err(syn::Error::new(
                self.keyword_span(),
                "conflicting error handling keyword",
            ))
        }
    }
}
