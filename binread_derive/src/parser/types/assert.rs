use core::convert::TryFrom;
use crate::parser::attrs;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, ExprLit, Lit, parse::Parse, spanned::Spanned};

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

impl <K: Parse + Spanned> TryFrom<attrs::AssertLike<K>> for Assert {
    type Error = syn::Error;

    fn try_from(value: attrs::AssertLike<K>) -> Result<Self, Self::Error> {
        let (cond, error) = {
            let mut args = value.fields.iter();

            let cond = if let Some(cond) = args.next() {
                cond
            } else {
                return Err(Self::Error::new(
                    value.ident.span(),
                    "`assert` requires a boolean expression as an argument"
                ));
            };

            let error = match args.next() {
                Some(Expr::Lit(ExprLit { lit: Lit::Str(message), .. })) => {
                    Some(Error::Message(quote! {
                        extern crate alloc;
                        alloc::format!(#message #(, #args)*)
                    }))
                },
                Some(error) => {
                    let mut extra_span = None::<proc_macro2::Span>;
                    for extra_arg in args {
                        let arg_span = extra_arg.span();
                        if let Some(span) = extra_span {
                            // This join will fail if the `proc_macro_span`
                            // feature is unavailable. Falling back to the
                            // `ident` span is better than doing nothing.
                            if let Some(new_span) = span.join(arg_span) {
                                extra_span = Some(new_span);
                            } else {
                                extra_span = Some(value.ident.span());
                                break;
                            }
                        } else {
                            extra_span = Some(arg_span);
                        }
                    }

                    if let Some(span) = extra_span {
                        return Err(Self::Error::new(span, "too many arguments"));
                    }

                    Some(Error::Error(error.to_token_stream()))
                },
                None => None,
            };

            (cond, error)
        };

        Ok(Self {
            condition: cond.into_token_stream(),
            consequent: error,
        })
    }
}
