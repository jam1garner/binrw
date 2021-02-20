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
        let (cond, err) = {
            let mut iter = value.fields.iter();
            let result = (iter.next(), iter.next());
            if iter.next().is_some() {
                return Err(Self::Error::new(
                    value.ident.span(),
                    "Too many arguments passed to assert"
                ));
            }
            result
        };

        Ok(Self(
            cond.into_token_stream(),
            err.map(ToTokens::into_token_stream)
        ))
    }
}

impl <T: TryInto<Assert, Error = syn::Error> + KeywordToken> TrySet<Vec<Assert>> for T {
    fn try_set(self, to: &mut Vec<Assert>) -> syn::Result<()> {
        to.push(self.try_into()?);
        Ok(())
    }
}
