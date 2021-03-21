use crate::parser::{attrs, KeywordToken, TrySet};
use proc_macro2::TokenStream;
use quote::ToTokens;

#[derive(Debug, Clone)]
pub(crate) enum PassedArgs {
    None,
    List(Vec<TokenStream>),
    Tuple(TokenStream),
}

impl Default for PassedArgs {
    fn default() -> Self {
        PassedArgs::None
    }
}

impl From<attrs::Args> for PassedArgs {
    fn from(args: attrs::Args) -> Self {
        Self::List(
            args.fields
                .iter()
                .map(ToTokens::into_token_stream)
                .collect(),
        )
    }
}

impl From<attrs::ArgsTuple> for PassedArgs {
    fn from(args: attrs::ArgsTuple) -> Self {
        Self::Tuple(args.value.into_token_stream())
    }
}

impl<T: Into<PassedArgs> + KeywordToken> TrySet<PassedArgs> for T {
    fn try_set(self, to: &mut PassedArgs) -> syn::Result<()> {
        if matches!(*to, PassedArgs::None) {
            *to = self.into();
            Ok(())
        } else {
            Err(syn::Error::new(
                self.keyword_span(),
                "conflicting args keyword",
            ))
        }
    }
}
