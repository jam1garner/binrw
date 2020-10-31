use proc_macro2::Span;

#[derive(Debug, Clone)]
pub struct SpannedValue<T> {
    span: Span,
    value: T,
}

impl<T> SpannedValue<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { span, value }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

impl<T> std::ops::Deref for SpannedValue<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Default> Default for SpannedValue<T> {
    fn default() -> Self {
        SpannedValue::new(
            Default::default(),
            Span::call_site()
        )
    }
}
