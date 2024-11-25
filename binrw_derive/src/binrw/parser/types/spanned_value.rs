use crate::meta_types::KeywordToken;
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

    #[cfg(feature = "verbose-backtrace")]
    pub(crate) fn into_value(self) -> T {
        self.value
    }

    pub(crate) fn span(&self) -> Span {
        self.span
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

impl<T: Into<To> + KeywordToken, To> From<T> for SpannedValue<To> {
    fn from(value: T) -> Self {
        let span = value.keyword_span();
        Self {
            value: value.into(),
            span,
        }
    }
}
