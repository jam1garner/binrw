//! Type definitions for wrappers which parse interleaved data.

use crate::io::{Read, Seek};
use crate::{BinRead, BinResult, ReadOptions};

use binrw_derive::BinrwNamedArgs;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;

/// A parser for data which consists of values of type `T` interleaved with
/// other values of type `P`.
///
/// To use this parser, you must specify the parsing strategy by setting the
/// `strategy` argument to either [`Separated`] or [`SeparatedTrailing`].
///
/// Consider using a `Vec<(T, P)>` or `(Vec<(T, P)>, Option<T>>)` instead if you
/// do not need the parsed data to be transformed into a structure of arrays.
///
/// # Examples
///
/// ```rust
/// # use binrw::{*, io::*};
/// use binrw::punctuated::{Punctuated, PunctuatedStrategy};
///
/// #[derive(BinRead)]
/// struct MyList {
///     #[br(args { strategy: PunctuatedStrategy::Separated, count: 3, inner: () })]
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

/// Strategies for the Punctuated parser
#[derive(Clone, PartialEq)]
pub enum PunctuatedStrategy {
    /// Parses values of type `T` separated by values of type `P` without a
    /// trailing separator value.
    Separated,
    /// Parses values of type `T` separated by values of type `P` with a
    /// trailing separator value.
    SeparatedTrailing,
}

/// Arguments passed to the binread impl for Punctuated
#[derive(BinrwNamedArgs, Clone)]
pub struct PunctuatedArgs<B> {
    /// The parser strategy
    pub strategy: PunctuatedStrategy,
    /// The number of elements to read.
    pub count: usize,

    /// Arguments to pass to the inner type
    #[named_args(try_optional)]
    pub inner: B,
}

impl<T: BinRead, P: BinRead<Args = ()>> BinRead for Punctuated<T, P> {
    type Args = PunctuatedArgs<T::Args>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        let mut data = Vec::with_capacity(args.count);
        let mut separators = Vec::with_capacity(args.count.max(1) - 1);

        for i in 0..args.count {
            data.push(T::read_options(reader, &options, args.inner.clone())?);
            if args.strategy == PunctuatedStrategy::SeparatedTrailing || i + 1 != args.count {
                separators.push(P::read_options(reader, options, ())?);
            }
        }

        Ok(Self { data, separators })
    }
}

impl<T: BinRead, P: BinRead> Punctuated<T, P> {
    /// Consumes this object, returning the data values while dropping the
    /// separator values.
    ///
    /// If you never use the separator values, consider using the [`pad_after`]
    /// directive to skip over data while parsing instead of reading it into
    /// memory and then discarding it.
    ///
    /// [`pad_after`]: crate::attribute#padding-and-alignment
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
