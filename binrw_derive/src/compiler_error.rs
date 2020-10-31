use proc_macro2::Span;

#[derive(Debug)]
pub enum CompileError {
    SpanError(SpanError),
    Syn(syn::Error),
}

#[derive(Debug)]
pub struct SpanError(pub Span, pub String);

impl SpanError {
    #[allow(dead_code)]
    pub fn new<E: Into<String>>(span: Span, err: E) -> Self {
        SpanError(span, err.into())
    }

    pub fn err<K, E: Into<String>>(span: Span, err: E) -> Result<K, Self> {
        Err(SpanError(span, err.into()))
    }
}

impl From<syn::Error> for CompileError {
    fn from(err: syn::Error) -> Self {
        CompileError::Syn(err)
    }
}

impl From<SpanError> for CompileError {
    fn from(err: SpanError) -> Self {
        CompileError::SpanError(err)
    }
}
