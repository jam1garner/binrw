use crate::{binrw::parser::attrs, meta_types::KeywordToken};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
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
    pub(crate) consequent: Option<Error>,
}

impl<K: Parse + Spanned + Token> TryFrom<attrs::AssertLike<K>> for Assert {
    type Error = syn::Error;

    fn try_from(value: attrs::AssertLike<K>) -> Result<Self, Self::Error> {
        let kw_span = value.keyword_span();
        let mut args = value.fields.iter();

        let condition = if let Some(cond) = args.next() {
            cond.into_token_stream()
        } else {
            return Err(Self::Error::new(
                kw_span,
                format!(
                    "{} requires a boolean expression as an argument",
                    value.dyn_display()
                ),
            ));
        };

        // ignores any alternative declaration of `self` in the condition, but asserts should be
        // simple so that shouldn't be a problem
        let mut condition_uses_self = false;
        let condition: TokenStream = condition.into_iter().map(|tt| {
            match tt {
                TokenTree::Ident(ref i) if i == "self" => {
                    condition_uses_self = true;
                    TokenTree::Ident(Ident::new("this", i.span()))
                }
                other => other,
            }
        }).collect();

        let consequent = match args.next() {
            Some(Expr::Lit(ExprLit {
                lit: Lit::Str(message),
                ..
            })) => Some(Error::Message(quote! {
                extern crate alloc;
                alloc::format!(#message #(, #args)*)
            })),
            Some(error) => {
                super::assert_all_args_consumed(args, value.keyword_span())?;
                Some(Error::Error(error.to_token_stream()))
            }
            None => None,
        };

        Ok(Self {
            kw_span,
            condition,
            condition_uses_self,
            consequent,
        })
    }
}
