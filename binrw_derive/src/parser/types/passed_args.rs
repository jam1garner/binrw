use super::SpannedValue;
use crate::parser::{attrs, meta_types::Enclosure, KeywordToken, TrySet};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::spanned::Spanned;

#[derive(Debug, Clone)]
pub(crate) enum PassedArgs {
    None,
    List(SpannedValue<Vec<TokenStream>>),
    Tuple(SpannedValue<TokenStream>),
    Named(SpannedValue<Vec<TokenStream>>),
}

impl PassedArgs {
    pub(crate) fn is_some(&self) -> bool {
        !matches!(self, Self::None)
    }
}

impl Default for PassedArgs {
    fn default() -> Self {
        PassedArgs::None
    }
}

impl From<attrs::Args> for PassedArgs {
    fn from(args: attrs::Args) -> Self {
        match args.list {
            Enclosure::Brace { fields, .. } => Self::Named(SpannedValue::new(
                fields
                    .into_iter()
                    .map(ToTokens::into_token_stream)
                    .collect(),
                args.ident.span(),
            )),
            Enclosure::Paren { fields, .. } => Self::List(SpannedValue::new(
                fields
                    .into_iter()
                    .map(ToTokens::into_token_stream)
                    .collect(),
                args.ident.span(),
            )),
        }
    }
}

impl From<attrs::ArgsRaw> for PassedArgs {
    fn from(args: attrs::ArgsRaw) -> Self {
        Self::Tuple(SpannedValue::new(
            args.value.into_token_stream(),
            args.ident.span(),
        ))
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
