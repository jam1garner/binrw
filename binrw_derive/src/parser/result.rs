#[derive(Debug)]
pub(crate) enum PartialResult<T, E> {
    Ok(T),
    Partial(T, E),
    Err(E),
}

impl<T, E> PartialResult<T, E> {
    #[cfg(test)]
    pub(crate) fn err(self) -> Option<E> {
        match self {
            PartialResult::Ok(_) => None,
            PartialResult::Partial(_, error) | PartialResult::Err(error) => Some(error),
        }
    }

    pub(crate) fn map<F, U>(self, f: F) -> PartialResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            PartialResult::Ok(value) => PartialResult::Ok(f(value)),
            PartialResult::Partial(value, error) => PartialResult::Partial(f(value), error),
            PartialResult::Err(error) => PartialResult::Err(error),
        }
    }

    pub(crate) fn ok(self) -> Option<T> {
        match self {
            PartialResult::Ok(value) | PartialResult::Partial(value, _) => Some(value),
            PartialResult::Err(_) => None,
        }
    }
}

impl<T, E: core::fmt::Debug> PartialResult<T, E> {
    #[cfg(test)]
    #[track_caller]
    pub(crate) fn unwrap(self) -> T {
        match self {
            PartialResult::Ok(value) => value,
            PartialResult::Partial(_, error) => panic!(
                "called `PartialResult::unwrap() on a `Partial` value: {:?}",
                &error
            ),
            PartialResult::Err(error) => panic!(
                "called `PartialResult::unwrap() on an `Err` value: {:?}",
                &error
            ),
        }
    }

    pub(crate) fn unwrap_tuple(self) -> (T, Option<E>) {
        match self {
            PartialResult::Ok(value) => (value, None),
            PartialResult::Partial(value, error) => (value, Some(error)),
            PartialResult::Err(error) => panic!(
                "called `PartialResult::unwrap_tuple() on an `Err` value: {:?}",
                &error
            ),
        }
    }
}

pub(crate) type ParseResult<T> = PartialResult<T, syn::Error>;
