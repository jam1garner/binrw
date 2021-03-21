use crate::parser::attrs;
use core::convert::TryFrom;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

#[derive(Debug, Clone)]
pub(crate) struct Condition {
    pub(crate) condition: TokenStream,
    pub(crate) alternate: TokenStream,
}

impl TryFrom<attrs::If> for Condition {
    type Error = syn::Error;

    fn try_from(value: attrs::If) -> Result<Self, Self::Error> {
        let mut args = value.fields.iter();

        let condition = if let Some(cond) = args.next() {
            cond.into_token_stream()
        } else {
            return Err(Self::Error::new(
                value.ident.span(),
                "`if` requires a boolean expression as an argument",
            ));
        };

        let alternate = args
            .next()
            .map_or_else(|| quote! { <_>::default() }, ToTokens::into_token_stream);

        super::assert_all_args_consumed(args, value.ident.span())?;

        Ok(Self {
            condition,
            alternate,
        })
    }
}
