use crate::{
    binrw::{codegen::sanitization::THIS, parser::attrs},
    meta_types::KeywordToken,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::fold::Fold;
use syn::{parse::Parse, spanned::Spanned, token::Token, Expr, ExprLit, Lit};

#[derive(Debug, Clone)]
pub(crate) enum Error {
    Message(TokenStream),
    Error(TokenStream),
}

#[derive(Debug, Clone)]
pub(crate) struct Assert {
    pub(crate) kw_span: Span,
    pub(crate) condition: TokenStream,
    /// `true` if the condition was written with `self`, in the [`condition`] it is replaced with
    /// `this`. This enables backwards compatibility with asserts that did not use `self`.
    pub(crate) condition_uses_self: bool,
    pub(crate) consequent: Error,
}

impl<K: Parse + Spanned + Token> TryFrom<attrs::AssertLike<K>> for Assert {
    type Error = syn::Error;

    fn try_from(value: attrs::AssertLike<K>) -> Result<Self, Self::Error> {
        let kw_span = value.keyword_span();
        let mut args = value.fields.iter();

        let Some(condition) = args.next() else {
            return Err(Self::Error::new(
                kw_span,
                format!(
                    "{} requires a boolean expression as an argument",
                    value.dyn_display()
                ),
            ));
        };

        // TODO: There should not be codegen in the parser
        let consequent = match args.next() {
            Some(Expr::Lit(ExprLit {
                lit: Lit::Str(message),
                ..
            })) => Error::Message(quote! {
                binrw::__private::format!(#message #(, #args)*)
            }),
            Some(error) => {
                super::assert_all_args_consumed(args, value.keyword_span())?;
                Error::Error(error.to_token_stream())
            }
            None => Error::Message({
                let condition = condition.to_token_stream().to_string();
                quote! {
                    binrw::__private::format!("assertion failed: `{}`", #condition)
                }
            }),
        };

        // ignores any alternative declaration of `self` in the condition, but
        // asserts should be simple so that shouldn't be a problem
        let mut self_replacer = ReplaceSelfWithThis { uses_self: false };
        let condition = self_replacer.fold_expr(condition.clone());

        Ok(Self {
            kw_span,
            condition: condition.into_token_stream(),
            condition_uses_self: self_replacer.uses_self,
            consequent,
        })
    }
}

struct ReplaceSelfWithThis {
    uses_self: bool,
}

impl Fold for ReplaceSelfWithThis {
    fn fold_ident(&mut self, i: Ident) -> Ident {
        if i == "self" {
            self.uses_self = true;
            THIS.to_ident(i.span())
        } else {
            i
        }
    }
}
