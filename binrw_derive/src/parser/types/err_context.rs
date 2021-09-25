use crate::parser::{keywords, meta_types::MetaList};
use core::convert::TryFrom;

#[derive(Debug, Clone)]
pub(crate) enum ErrContext {
    Context(syn::Expr),
    Format(syn::LitStr, Vec<syn::Expr>),
}

fn is_lit_str(expr: &syn::Expr) -> bool {
    matches!(
        expr,
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(_),
            ..
        })
    )
}

fn as_lit_str(expr: &syn::Expr) -> syn::LitStr {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit_str),
        ..
    }) = expr
    {
        lit_str.clone()
    } else {
        panic!("Not a string literal")
    }
}

impl TryFrom<MetaList<keywords::err_context, syn::Expr>> for ErrContext {
    type Error = syn::Error;

    fn try_from(value: MetaList<keywords::err_context, syn::Expr>) -> Result<Self, Self::Error> {
        match value.fields.len() {
            0 => Err(syn::Error::new_spanned(
                value.ident,
                "err_context requires a value but none were given",
            )),
            // format string
            _ if is_lit_str(&value.fields[0]) => {
                let format = as_lit_str(&value.fields[0]);
                Ok(ErrContext::Format(
                    format,
                    value.fields.into_iter().skip(1).collect(),
                ))
            }
            // payload
            1 => Ok(ErrContext::Context(value.fields[0].clone())),
            _ => Err(syn::Error::new_spanned(
                &value.fields[0],
                "format string expected",
            )),
        }
    }
}
