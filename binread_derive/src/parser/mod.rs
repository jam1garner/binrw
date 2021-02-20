#[macro_use]
pub(crate) mod macros;
mod attrs;
mod field_level_attrs;
mod keywords;
pub(crate) mod meta_types;
mod top_level_attrs;
mod types;

pub(crate) use field_level_attrs::*;
use proc_macro2::Span;
use meta_types::MetaAttrList;
use syn::{spanned::Spanned, token::Token};
pub(crate) use top_level_attrs::*;
pub(crate) use types::*;

pub(crate) trait FromAttrs<Attr: syn::parse::Parse> {
    fn try_from_attrs(attrs: &[syn::Attribute]) -> syn::Result<Self> where Self: Default + Sized {
        Self::set_from_attrs(Self::default(), attrs)
    }

    fn set_from_attrs(mut self, attrs: &[syn::Attribute]) -> syn::Result<Self> where Self: Sized {
        #[allow(clippy::filter_map)]
        let attrs = attrs
            .iter()
            .filter(|attr| attr.path.is_ident("br") || attr.path.is_ident("binread"))
            .flat_map(|attr| {
                match syn::parse2::<MetaAttrList<Attr>>(attr.tokens.clone()) {
                    Ok(list) => either::Either::Right(list.into_iter().map(Ok)),
                    Err(err) => either::Either::Left(core::iter::once(Err(err))),
                }
            });

        let mut all_errors = None::<syn::Error>;
        for attr in attrs {
            let result = match attr {
                Ok(attr) => self.try_set_attr(attr),
                Err(e) => Err(e),
            };

            if let Err(parse_error) = result {
                if let Some(error) = &mut all_errors {
                    error.combine(parse_error);
                } else {
                    all_errors = Some(parse_error);
                }
            }
        }

        match all_errors {
            Some(error) => Err(error),
            None => Ok(self),
        }
    }

    fn try_set_attr(&mut self, attr: Attr) -> syn::Result<()>;
}

pub(crate) trait FromField {
    type In;

    fn from_field(field: &Self::In) -> syn::Result<Self> where Self: Sized;
}

pub(crate) trait FromInput<Attr: syn::parse::Parse>: FromAttrs<Attr> {
    type Field: FromField + 'static;

    fn from_input<'input>(attrs: &'input [syn::Attribute], fields: impl Iterator<Item = &'input <Self::Field as FromField>::In>) -> syn::Result<Self> where Self: Sized + Default {
        // TODO: This probably should return an incomplete object + error so
        // that all field validation can occur; currently if the parent object
        // has an error then any validation in push_field will not occur.
        let (mut this, mut all_errors) = match Self::try_from_attrs(attrs) {
            Ok(this) => (Some(this), None),
            Err(all_errors) => (None, Some(all_errors)),
        };

        for field in fields {
            let field_error = match Self::Field::from_field(field) {
                Ok(field) => match &mut this {
                    Some(this) => this.push_field(field),
                    None => Ok(()),
                },
                Err(field_error) => Err(field_error),
            };

            if let Err(field_error) = field_error {
                match &mut all_errors {
                    Some(all_errors) => all_errors.combine(field_error),
                    None => all_errors = Some(field_error),
                }
            }
        }

        all_errors.map_or_else(|| Ok(this.unwrap()), Err)
    }

    fn push_field(&mut self, field: Self::Field) -> syn::Result<()>;
}

pub(crate) trait KeywordToken {
    type Token: Token;

    fn display() -> &'static str {
        <Self::Token as Token>::display()
    }

    fn dyn_display(&self) -> &'static str {
        Self::display()
    }

    fn keyword_span(&self) -> Span;
}

impl <T: Token + Spanned> KeywordToken for T {
    type Token = T;

    fn keyword_span(&self) -> Span {
        self.span()
    }
}

pub(crate) trait TrySet<T> {
    fn try_set(self, to: &mut T) -> syn::Result<()>;
}

impl <T: KeywordToken> TrySet<bool> for T {
    fn try_set(self, to: &mut bool) -> syn::Result<()> {
        if *to {
            Err(syn::Error::new(self.keyword_span(), format!("conflicting {} keyword", self.dyn_display())))
        } else {
            *to = true;
            Ok(())
        }
    }
}

impl <T: Into<To> + KeywordToken, To> TrySet<Option<To>> for T {
    fn try_set(self, to: &mut Option<To>) -> syn::Result<()> {
        if to.is_none() {
            *to = Some(self.into());
            Ok(())
        } else {
            Err(syn::Error::new(self.keyword_span(), format!("conflicting {} keyword", self.dyn_display())))
        }
    }
}
