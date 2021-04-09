use crate::parser::{attrs, meta_types::IdentTypeMaybeDefault, KeywordToken, TrySet};

use syn::{Ident, Type};

#[derive(Debug, Clone)]
pub(crate) enum Imports {
    None,
    Raw(Ident, Box<Type>),
    List(Vec<Ident>, Vec<Type>),
    Named(Vec<IdentTypeMaybeDefault>),
}

impl Default for Imports {
    fn default() -> Self {
        Imports::None
    }
}

impl From<attrs::ImportNamed> for Imports {
    fn from(value: attrs::ImportNamed) -> Self {
        if value.fields.is_empty() {
            Self::None
        } else {
            Self::Named(value.fields.iter().cloned().collect())
        }
    }
}

impl From<attrs::ImportRaw> for Imports {
    fn from(value: attrs::ImportRaw) -> Self {
        Imports::Raw(value.value.ident, value.value.ty.into())
    }
}

impl<T: Into<Imports> + KeywordToken> TrySet<Imports> for T {
    fn try_set(self, to: &mut Imports) -> syn::Result<()> {
        if matches!(*to, Imports::None) {
            *to = self.into();
            Ok(())
        } else {
            Err(syn::Error::new(
                self.keyword_span(),
                "conflicting import keyword",
            ))
        }
    }
}
