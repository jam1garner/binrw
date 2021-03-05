//! A module for [`Punctuated<T, P>`](Punctuated), a series of items to parse of type T separated
//! by punction of type `P`.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::fmt;
use crate::io::{Read, Seek};
use crate::{BinRead, ReadOptions, BinResult};

/// A type for seperated data. Since parsing for this type is ambiguous, you must manually specify
/// a parser using the `parse_with` attribute.
///
/// ## Example
///
/// ```rust
/// # use binread::{*, io::*};
/// use binread::punctuated::Punctuated;
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
/// # assert_eq!(y.x.seperators, vec![0, 1]);
/// ```
pub struct Punctuated<T: BinRead, P: BinRead> {
    data: Vec<T>,
    pub seperators: Vec<P>,
}

impl<C: Copy + 'static, T: BinRead<Args = C>, P: BinRead<Args = ()>> Punctuated<T, P> {
    /// A parser for values seperated by another value, with no trailing punctuation.
    ///
    /// Requires a specified count.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use binread::{*, io::*};
    /// use binread::punctuated::Punctuated;
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
    /// # assert_eq!(y.x.seperators, vec![0, 1]);
    /// ```
    pub fn separated<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: C) -> BinResult<Self> {
        let count = match options.count {
            Some(x) => x,
            None => panic!("Missing count for Punctuated"),
        };

        let mut data = Vec::with_capacity(count);
        let mut seperators = Vec::with_capacity(count.max(1) - 1);

        for i in 0..count {
            data.push(T::read_options(reader, &options, args)?);
            if i + 1 != count {
                seperators.push(P::read_options(reader, options, ())?);
            }
        }

        Ok(Self { data, seperators })
    }

    /// A parser for values seperated by another value, with trailing punctuation.
    ///
    /// Requires a specified count.
    pub fn separated_trailing<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: C) -> BinResult<Self> {
        let count = match options.count {
            Some(x) => x,
            None => panic!("Missing count for Punctuated"),
        };

        let mut data = Vec::with_capacity(count);
        let mut seperators = Vec::with_capacity(count);

        for _ in 0..count {
            data.push(T::read_options(reader, &options, args)?);
            seperators.push(P::read_options(reader, options, ())?);
        }

        Ok(Self { data, seperators })
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
