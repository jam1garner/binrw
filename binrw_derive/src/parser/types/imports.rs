use crate::parser::{
    meta_types::{Enclosure, IdentPatType, IdentTypeMaybeDefault},
    read, KeywordToken, TrySet,
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

fn imports_from_attr(list: &Enclosure<IdentPatType, IdentTypeMaybeDefault>) -> Imports {
    match list {
        Enclosure::Paren { fields, .. } => {
            if fields.is_empty() {
                Imports::None
            } else {
                let (idents, tys) = fields
                    .iter()
                    .cloned()
                    .map(|field| (field.ident, field.ty))
                    .unzip();
                Imports::List(idents, tys)
            }
        }
        Enclosure::Brace { fields, .. } => {
            if fields.is_empty() {
                Imports::None
            } else {
                Imports::Named(fields.iter().cloned().collect())
            }
        }
    }
}

impl From<read::attrs::Import> for Imports {
    fn from(value: read::attrs::Import) -> Self {
        imports_from_attr(&value.list)
    }
}

impl From<read::attrs::ImportRaw> for Imports {
    fn from(value: read::attrs::ImportRaw) -> Self {
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
