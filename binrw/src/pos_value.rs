use crate::{
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, Endian,
};
use core::fmt;

/// A wrapper that stores a value’s position alongside the value.
/// Serializing a `PosValue` will ignore the `pos` field.
///
/// # Examples
///
/// ```
/// use binrw::{BinRead, PosValue, BinReaderExt, io::Cursor};
///
/// #[derive(BinRead)]
/// struct MyType {
///     a: u16,
///     b: PosValue<u8>
/// }
///
/// let val = Cursor::new(b"\xFF\xFE\xFD").read_be::<MyType>().unwrap();
/// assert_eq!(val.b.pos, 2);
/// assert_eq!(*val.b, 0xFD);
/// ```
pub struct PosValue<T> {
    /// The read value.
    pub val: T,

    /// The byte position of the start of the value.
    pub pos: u64,
}

impl<T: BinRead> BinRead for PosValue<T> {
    type Args<'a> = T::Args<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let pos = reader.stream_position()?;

        Ok(PosValue {
            pos,
            val: T::read_options(reader, endian, args)?,
        })
    }
}

impl<T: BinWrite> BinWrite for PosValue<T> {
    type Args<'a> = T::Args<'a>;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        self.val.write_options(writer, endian, args)
    }
}

impl<T> core::ops::Deref for PosValue<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.val
    }
}

impl<T> core::ops::DerefMut for PosValue<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.val
    }
}

impl<T: fmt::Debug> fmt::Debug for PosValue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.val.fmt(f)
    }
}

impl<T: Clone> Clone for PosValue<T> {
    fn clone(&self) -> Self {
        Self {
            val: self.val.clone(),
            pos: self.pos,
        }
    }
}

impl<U, T: PartialEq<U>> PartialEq<U> for PosValue<T> {
    fn eq(&self, other: &U) -> bool {
        self.val == *other
    }
}

impl<T: Default> Default for PosValue<T> {
    fn default() -> Self {
        Self {
            val: Default::default(),
            pos: Default::default(),
        }
    }
}

impl<T> From<T> for PosValue<T> {
    fn from(val: T) -> Self {
        Self {
            val,
            pos: Default::default(),
        }
    }
}
