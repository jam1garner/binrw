//! Type definitions for wrappers which parse interleaved data.

use crate::{BinRead, BinResult, VecArgs};
use alloc::vec::Vec;
use core::fmt;

/// A parser for data which consists of values of type `T` interleaved with
/// other values of type `P`.
///
/// To use this parser, you must specify the parsing strategy by selecting
/// either [`separated()`] or [`separated_trailing()`] using [`parse_with`].
///
/// [`separated()`]: Self::separated
/// [`separated_trailing()`]: Self::separated_trailing
/// [`parse_with`]: crate::docs::attribute#custom-parserswriters
///
/// Consider using a `Vec<(T, P)>` or `(Vec<(T, P)>, Option<T>>)` instead if you
/// do not need the parsed data to be transformed into a structure of arrays.
///
/// # Examples
///
/// ```
/// # use binrw::{prelude::*, io::Cursor};
/// use binrw::punctuated::Punctuated;
///
/// #[derive(BinRead)]
/// struct MyList {
///     #[br(parse_with = Punctuated::separated)]
///     #[br(count = 3)]
///     x: Punctuated<u16, u8>,
/// }
///
/// # let mut x = Cursor::new(b"\0\x03\0\0\x02\x01\0\x01");
/// # let y: MyList = x.read_be().unwrap();
/// # assert_eq!(*y.x, vec![3, 2, 1]);
/// # assert_eq!(y.x.separators, vec![0, 1]);
/// ```
pub struct Punctuated<T: BinRead, P: BinRead> {
    /// The data values.
    data: Vec<T>,

    /// The separator values.
    pub separators: Vec<P>,
}

impl<T: BinRead, P: BinRead<Args = ()>> Punctuated<T, P> {
    /// Parses values of type `T` separated by values of type `P` without a
    /// trailing separator value.
    ///
    /// Requires a count to be passed via `#[br(count)]`.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use binrw::{prelude::*, io::Cursor};
    /// use binrw::punctuated::Punctuated;
    ///
    /// #[derive(BinRead)]
    /// struct MyList {
    ///     #[br(parse_with = Punctuated::separated)]
    ///     #[br(count = 3)]
    ///     x: Punctuated<u16, u8>,
    /// }
    ///
    /// # let mut x = Cursor::new(b"\0\x03\0\0\x02\x01\0\x01");
    /// # let y: MyList = x.read_be().unwrap();
    /// # assert_eq!(*y.x, vec![3, 2, 1]);
    /// # assert_eq!(y.x.separators, vec![0, 1]);
    /// ```
    #[crate::parser(reader, endian)]
    pub fn separated(args: VecArgs<T::Args>, ...) -> BinResult<Self> {
        let mut data = Vec::with_capacity(args.count);
        let mut separators = Vec::with_capacity(args.count.max(1) - 1);

        for i in 0..args.count {
            data.push(T::read_options(reader, endian, args.inner.clone())?);
            if i + 1 != args.count {
                separators.push(P::read_options(reader, endian, ())?);
            }
        }

        Ok(Self { data, separators })
    }

    /// Parses values of type `T` interleaved with values of type `P`, including
    /// a trailing `P`.
    ///
    /// Requires a count to be passed via `#[br(count)]`.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[crate::parser(reader, endian)]
    pub fn separated_trailing(args: VecArgs<T::Args>, ...) -> BinResult<Self> {
        let mut data = Vec::with_capacity(args.count);
        let mut separators = Vec::with_capacity(args.count);

        for _ in 0..args.count {
            data.push(T::read_options(reader, endian, args.inner.clone())?);
            separators.push(P::read_options(reader, endian, ())?);
        }

        Ok(Self { data, separators })
    }

    /// Consumes this object, returning the data values while dropping the
    /// separator values.
    ///
    /// If you never use the separator values, consider using the [`pad_after`]
    /// directive to skip over data while parsing instead of reading it into
    /// memory and then discarding it.
    ///
    /// [`pad_after`]: crate::docs::attribute#padding-and-alignment
    #[must_use]
    pub fn into_values(self) -> Vec<T> {
        self.data
    }
}

impl<T: BinRead + fmt::Debug, P: BinRead> fmt::Debug for Punctuated<T, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

impl<T: BinRead, P: BinRead> core::ops::Deref for Punctuated<T, P> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: BinRead, P: BinRead> core::ops::DerefMut for Punctuated<T, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
