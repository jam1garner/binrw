#[derive(Debug)]
pub(crate) struct SpannedValue<T> {
    span: proc_macro2::Span,
    value: T,
}

impl<T> SpannedValue<T> {
    pub fn new(value: T, span: proc_macro2::Span) -> Self {
        Self { span, value }
    }

    pub fn span(&self) -> proc_macro2::Span {
        self.span
    }
}

impl<T> std::ops::Deref for SpannedValue<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
