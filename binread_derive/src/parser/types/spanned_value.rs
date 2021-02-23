use proc_macro2::Span;

#[derive(Debug, Clone)]
pub(crate) struct SpannedValue<T> {
    value: T,
    span: Span,
}

impl <T> SpannedValue<T> {
    pub(crate) fn new(value: T, span: Span) -> Self {
        SpannedValue { value, span }
    }
}

impl <T: Default> Default for SpannedValue<T> {
    fn default() -> Self {
        SpannedValue::new(Default::default(), Span::call_site())
    }
}

impl <T> core::ops::Deref for SpannedValue<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl <T> core::ops::DerefMut for SpannedValue<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl <T> syn::spanned::Spanned for SpannedValue<T> {
    fn span(&self) -> Span {
        self.span
    }
}

impl <T: Into<To> + crate::parser::KeywordToken, To> From<T> for SpannedValue<To> {
    fn from(value: T) -> Self {
        let span = value.keyword_span();
        Self { value: value.into(), span }
    }
}
