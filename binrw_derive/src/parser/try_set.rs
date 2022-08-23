use super::KeywordToken;

pub(crate) trait TrySet<T> {
    fn try_set(self, to: &mut T) -> syn::Result<()>;
}

// TODO: This sucks
pub(crate) enum TrySetError {
    Infallible,
    Syn(syn::Error),
}

impl From<core::convert::Infallible> for TrySetError {
    fn from(_: core::convert::Infallible) -> Self {
        Self::Infallible
    }
}

impl From<syn::Error> for TrySetError {
    fn from(error: syn::Error) -> Self {
        Self::Syn(error)
    }
}

impl<T: core::convert::TryInto<To, Error = E> + KeywordToken, E: Into<TrySetError>, To>
    TrySet<Option<To>> for T
{
    fn try_set(self, to: &mut Option<To>) -> syn::Result<()> {
        if to.is_none() {
            *to = Some(self.try_into().map_err(|error| match error.into() {
                TrySetError::Infallible => unreachable!(),
                TrySetError::Syn(error) => error,
            })?);
            Ok(())
        } else {
            Err(syn::Error::new(
                self.keyword_span(),
                format!("conflicting {} keyword", self.dyn_display()),
            ))
        }
    }
}

impl<T: core::convert::TryInto<To, Error = syn::Error> + KeywordToken, To> TrySet<Vec<To>> for T {
    fn try_set(self, to: &mut Vec<To>) -> syn::Result<()> {
        to.push(self.try_into()?);
        Ok(())
    }
}
