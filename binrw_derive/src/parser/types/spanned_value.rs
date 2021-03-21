use crate::parser::{KeywordToken, TrySet};
use proc_macro2::Span;

#[derive(Debug, Clone)]
pub(crate) struct SpannedValue<T> {
    value: T,
    span: Span,
}

impl<T> SpannedValue<T> {
    pub(crate) fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

impl<T: Default> Default for SpannedValue<T> {
    fn default() -> Self {
        Self {
            value: <_>::default(),
            span: proc_macro2::Span::call_site(),
        }
    }
}

impl<T> AsRef<T> for SpannedValue<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> core::ops::Deref for SpannedValue<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

// It is not possible to implement this *and* ToTokens because syn has a generic
// implementation of Spanned for all ToTokens
impl<T> syn::spanned::Spanned for SpannedValue<T> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<T: Into<To> + KeywordToken, To> From<T> for SpannedValue<To> {
    fn from(value: T) -> Self {
        let span = value.keyword_span();
        Self {
            value: value.into(),
            span,
        }
    }
}

// TODO: This really should not be necessary but there are some really bad
// generic trait conflicts when trying to just implement `From`.
impl<T: KeywordToken> TrySet<SpannedValue<bool>> for T {
    fn try_set(self, to: &mut SpannedValue<bool>) -> syn::Result<()> {
        if to.value {
            Err(syn::Error::new(
                self.keyword_span(),
                format!("conflicting {} keyword", self.dyn_display()),
            ))
        } else {
            to.span = self.keyword_span();
            to.value = true;
            Ok(())
        }
    }
}
