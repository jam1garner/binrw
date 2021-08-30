use crate::parser::{
    read::attrs,
    meta_types::{Enclosure, IdentTypeMaybeDefault},
    KeywordToken, TrySet,
};

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

impl From<attrs::Import> for Imports {
    fn from(value: attrs::Import) -> Self {
        match &value.list {
            Enclosure::Paren { fields, .. } => {
                if fields.is_empty() {
                    Self::None
                } else {
                    let (idents, tys) = fields
                        .iter()
                        .cloned()
                        .map(|field| (field.ident, field.ty))
                        .unzip();
                    Self::List(idents, tys)
                }
            }
            Enclosure::Brace { fields, .. } => {
                if fields.is_empty() {
                    Self::None
                } else {
                    Self::Named(fields.iter().cloned().collect())
                }
            }
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
