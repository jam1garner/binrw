use crate::{binrw::parser::attrs, meta_types::KeywordToken};
use syn::{punctuated::Punctuated, Token, WherePredicate};

use super::SpannedValue;

pub(crate) type Bound = Option<SpannedValue<Inner>>;

#[derive(Clone, Debug)]
pub(crate) struct Inner(Punctuated<WherePredicate, Token![,]>);

impl Inner {
    pub(crate) fn predicates(&self) -> &Punctuated<WherePredicate, Token![,]> {
        &self.0
    }
}

impl TryFrom<attrs::Bound> for SpannedValue<Inner> {
    type Error = syn::Error;

    fn try_from(bound: attrs::Bound) -> Result<Self, Self::Error> {
        let kw_span = bound.keyword_span();
        Ok(Self::new(Inner(bound.fields), kw_span))
    }
}
