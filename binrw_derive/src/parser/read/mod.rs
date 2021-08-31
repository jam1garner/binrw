pub(crate) mod attrs;
mod field_level_attrs;
mod top_level_attrs;

pub(crate) use field_level_attrs::*;
pub(crate) use top_level_attrs::*;

use super::meta_types::MetaAttrList;
use super::{combine_error, FromField, ParseResult};

pub(crate) fn is_binread_attr(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("br") || attr.path.is_ident("brw") || attr.path.is_ident("binread")
}

pub(crate) trait FromAttrs<Attr: syn::parse::Parse> {
    fn try_from_attrs(attrs: &[syn::Attribute]) -> ParseResult<Self>
    where
        Self: Default + Sized,
    {
        Self::set_from_attrs(Self::default(), attrs)
    }

    fn set_from_attrs(mut self, attrs: &[syn::Attribute]) -> ParseResult<Self>
    where
        Self: Sized,
    {
        let attrs = attrs
            .iter()
            .filter(|attr| is_binread_attr(attr))
            .flat_map(
                |attr| match syn::parse2::<MetaAttrList<Attr>>(attr.tokens.clone()) {
                    Ok(list) => list.into_iter().map(Ok).collect::<Vec<_>>().into_iter(),
                    Err(err) => core::iter::once(Err(err)).collect::<Vec<_>>().into_iter(),
                },
            );

        let mut all_errors = None::<syn::Error>;
        for attr in attrs {
            let result = match attr {
                Ok(attr) => self.try_set_attr(attr),
                Err(e) => Err(e),
            };

            if let Err(parse_error) = result {
                combine_error(&mut all_errors, parse_error);
            }
        }

        // https://github.com/rust-lang/rust-clippy/issues/5822
        #[allow(clippy::option_if_let_else)]
        if let Some(error) = all_errors {
            ParseResult::Partial(self, error)
        } else {
            ParseResult::Ok(self)
        }
    }

    fn try_set_attr(&mut self, attr: Attr) -> syn::Result<()>;
}

pub(crate) trait FromInput<Attr: syn::parse::Parse>: FromAttrs<Attr> {
    type Field: FromField + 'static;

    fn from_input<'input>(
        attrs: &'input [syn::Attribute],
        fields: impl Iterator<Item = &'input <Self::Field as FromField>::In>,
    ) -> ParseResult<Self>
    where
        Self: Sized + Default,
    {
        let (mut this, mut all_errors) = Self::try_from_attrs(attrs).unwrap_tuple();

        for (index, field) in fields.enumerate() {
            let (field, mut field_error) = Self::Field::from_field(field, index).unwrap_tuple();
            if field_error.is_none() {
                field_error = this.push_field(field).err();
            }

            if let Some(field_error) = field_error {
                combine_error(&mut all_errors, field_error);
            }
        }

        if let Err(validation_error) = this.validate() {
            combine_error(&mut all_errors, validation_error);
        }

        // https://github.com/rust-lang/rust-clippy/issues/5822
        #[allow(clippy::option_if_let_else)]
        if let Some(error) = all_errors {
            ParseResult::Partial(this, error)
        } else {
            ParseResult::Ok(this)
        }
    }

    fn push_field(&mut self, field: Self::Field) -> syn::Result<()>;

    fn validate(&self) -> syn::Result<()> {
        Ok(())
    }
}
