#[derive(Debug, Eq, PartialEq)]
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
                "called `PartialResult::unwrap()` on a `Partial` value: {:?}",
                &error
            ),
            PartialResult::Err(error) => panic!(
                "called `PartialResult::unwrap()` on an `Err` value: {:?}",
                &error
            ),
        }
    }

    pub(crate) fn unwrap_tuple(self) -> (T, Option<E>) {
        match self {
            PartialResult::Ok(value) => (value, None),
            PartialResult::Partial(value, error) => (value, Some(error)),
            PartialResult::Err(error) => panic!(
                "called `PartialResult::unwrap_tuple()` on an `Err` value: {:?}",
                &error
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct Pass;

    #[derive(Debug, Eq, PartialEq)]
    struct Error;

    #[test]
    #[cfg_attr(coverage_nightly, no_coverage)]
    fn err() {
        assert_eq!(PartialResult::<_, Error>::Ok(Pass).err(), None);
        assert_eq!(PartialResult::Partial(Pass, Error).err(), Some(Error));
        assert_eq!(PartialResult::<Pass, _>::Err(Error).err(), Some(Error));
    }

    #[test]
    #[cfg_attr(coverage_nightly, no_coverage)]
    fn map() {
        assert_eq!(
            PartialResult::<_, Error>::Ok(()).map(|()| Pass),
            PartialResult::Ok(Pass)
        );
        assert_eq!(
            PartialResult::Partial((), Error).map(|()| Pass),
            PartialResult::Partial(Pass, Error)
        );
        assert_eq!(
            PartialResult::<(), _>::Err(Error).map(|()| Pass),
            PartialResult::Err(Error)
        );
    }

    #[test]
    #[cfg_attr(coverage_nightly, no_coverage)]
    fn ok() {
        assert_eq!(PartialResult::<_, Error>::Ok(Pass).ok(), Some(Pass));
        assert_eq!(PartialResult::Partial(Pass, Error).ok(), Some(Pass));
        assert_eq!(PartialResult::<Pass, _>::Err(Error).ok(), None);
    }

    #[test]
    fn unwrap() {
        assert_eq!(PartialResult::<_, Error>::Ok(Pass).unwrap(), Pass);
    }

    #[test]
    #[cfg_attr(coverage_nightly, no_coverage)]
    #[should_panic(expected = "called `PartialResult::unwrap()` on an `Err` value")]
    fn unwrap_err() {
        PartialResult::<Pass, _>::Err(Error).unwrap();
    }

    #[test]
    #[cfg_attr(coverage_nightly, no_coverage)]
    #[should_panic(expected = "called `PartialResult::unwrap()` on a `Partial` value")]
    fn unwrap_partial() {
        PartialResult::Partial(Pass, Error).unwrap();
    }

    #[test]
    #[cfg_attr(coverage_nightly, no_coverage)]
    fn unwrap_tuple() {
        assert_eq!(
            PartialResult::<_, Error>::Ok(Pass).unwrap_tuple(),
            (Pass, None)
        );
        assert_eq!(
            PartialResult::Partial(Pass, Error).unwrap_tuple(),
            (Pass, Some(Error))
        );
    }

    #[test]
    #[cfg_attr(coverage_nightly, no_coverage)]
    #[should_panic(expected = "called `PartialResult::unwrap_tuple()` on an `Err` value")]
    fn unwrap_tuple_err() {
        PartialResult::<Pass, _>::Err(Error).unwrap_tuple();
    }
}
