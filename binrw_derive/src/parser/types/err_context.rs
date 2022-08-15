use crate::parser::{keywords, meta_types::MetaList};
use core::convert::TryFrom;

#[derive(Debug, Clone)]
pub(crate) enum ErrContext {
    Context(Box<syn::Expr>),
    Format(syn::LitStr, Vec<syn::Expr>),
}

impl TryFrom<MetaList<keywords::err_context, syn::Expr>> for ErrContext {
    type Error = syn::Error;

    fn try_from(value: MetaList<keywords::err_context, syn::Expr>) -> Result<Self, Self::Error> {
        if value.fields.is_empty() {
            Err(syn::Error::new_spanned(
                value.ident,
                "err_context requires a value but none were given",
            ))
        } else if let Some(format) = lit_str(&value.fields[0]) {
            Ok(ErrContext::Format(
                format.clone(),
                value.fields.into_iter().skip(1).collect(),
            ))
        } else if value.fields.len() == 1 {
            Ok(ErrContext::Context(Box::new(value.fields[0].clone())))
        } else {
            Err(syn::Error::new_spanned(
                &value.fields[0],
                "format string expected",
            ))
        }
    }
}

fn lit_str(expr: &syn::Expr) -> Option<&syn::LitStr> {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit_str),
        ..
    }) = expr
    {
        Some(lit_str)
    } else {
        None
    }
}
