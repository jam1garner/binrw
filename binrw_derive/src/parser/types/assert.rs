use crate::parser::{attrs, KeywordToken};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, spanned::Spanned, token::Token, Expr, ExprLit, Lit};

#[derive(Debug, Clone)]
pub(crate) enum Error {
    Message(TokenStream),
    Error(TokenStream),
}

#[derive(Debug, Clone)]
pub(crate) struct Assert {
    pub(crate) condition: TokenStream,
    pub(crate) consequent: Option<Error>,
}

impl<K: Parse + Spanned + Token> TryFrom<attrs::AssertLike<K>> for Assert {
    type Error = syn::Error;

    fn try_from(value: attrs::AssertLike<K>) -> Result<Self, Self::Error> {
        let mut args = value.fields.iter();

        let condition = if let Some(cond) = args.next() {
            cond.into_token_stream()
        } else {
            return Err(Self::Error::new(
                value.keyword_span(),
                format!(
                    "{} requires a boolean expression as an argument",
                    value.dyn_display()
                ),
            ));
        };

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
            condition,
            consequent,
        })
    }
}
