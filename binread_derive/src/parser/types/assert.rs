use core::convert::{TryFrom, TryInto};
use crate::parser::{KeywordToken, TrySet, attrs};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::Parse, spanned::Spanned};

#[derive(Debug, Clone)]
pub(crate) struct Assert(pub TokenStream, pub Option<TokenStream>);

impl <K: Parse + Spanned> TryFrom<attrs::AssertLike<K>> for Assert {
    type Error = syn::Error;

    fn try_from(value: attrs::AssertLike<K>) -> Result<Self, Self::Error> {
        let (cond, message) = {
            let mut args = value.fields.iter();

            let cond = if let Some(cond) = args.next() {
                cond
            } else {
                return Err(Self::Error::new(
                    value.ident.span(),
                    "`assert` requires a boolean expression as an argument"
                ));
            };

            let message = args.next();

            // TODO: This should work like `assert!` and accept formatting
            // arguments instead of rejecting
            let mut extra_span = None::<proc_macro2::Span>;
            for extra_arg in args {
                let arg_span = extra_arg.span();
                if let Some(span) = extra_span {
                    // This join will fail if the `proc_macro_span` feature is
                    // unavailable. Falling back to the `ident` span is better
                    // than doing nothing.
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

            (cond, message)
        };

        Ok(Self(
            cond.into_token_stream(),
            message.map(ToTokens::into_token_stream)
        ))
    }
}

impl <T: TryInto<Assert, Error = syn::Error> + KeywordToken> TrySet<Vec<Assert>> for T {
    fn try_set(self, to: &mut Vec<Assert>) -> syn::Result<()> {
        to.push(self.try_into()?);
        Ok(())
    }
}
