use crate::io::{Read, Seek};
use crate::{Endian, Error, ReadOptions};

/// A Result for any binread function that can return an error
pub type BinResult<T> = core::result::Result<T, Error>;

/// A `BinRead` trait allows reading a structure from anything that implements [`io::Read`](crate::io::Read) and [`io::Seek`](crate::io::Seek)
/// BinRead is implemented on the type to be read out of the given reader
pub trait BinRead: Sized + 'static {
    /// The type of arguments needed to be supplied in order to read this type, usually a tuple.
    ///
    /// **NOTE:** For types that don't require any arguments, use the unit (`()`) type. This will allow [`read`](crate::BinRead::read) to be used.
    type Args: Clone;

    /// Read the type from the reader while assuming no arguments have been passed
    fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        Self::Args: Default,
    {
        Self::read_options(reader, &ReadOptions::default(), Self::Args::default())
    }

    /// Read the type from the reader using the specified arguments
    fn read_args<R: Read + Seek>(reader: &mut R, args: Self::Args) -> BinResult<Self> {
        Self::read_options(reader, &ReadOptions::default(), args)
    }

    /// Read the type from the reader
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self>;

    fn after_parse<R: Read + Seek>(
        &mut self,
        _: &mut R,
        _: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<()> {
        Ok(())
    }
}

/// An extension trait for [`io::Read`](crate::io::Read) to provide methods for reading a value directly
///
/// ## Example
/// ```rust
/// use binrw::prelude::*; // BinReadExt is in the prelude
/// use binrw::endian::LE;
/// use binrw::io::Cursor;
///
/// let mut reader = Cursor::new(b"\x07\0\0\0\xCC\0\0\x05");
/// let x: u32 = reader.read_le().unwrap();
/// let y: u16 = reader.read_type(LE).unwrap();
/// let z = reader.read_be::<u16>().unwrap();
///
/// assert_eq!((x, y, z), (7u32, 0xCCu16, 5u16));
/// ```
pub trait BinReaderExt: Read + Seek + Sized {
    /// Read the given type from the reader using the given endianness.
    fn read_type<T: BinRead>(&mut self, endian: Endian) -> BinResult<T>
    where
        T::Args: Default,
    {
        self.read_type_args(endian, T::Args::default())
    }

    /// Read the given type from the reader with big endian byte order
    fn read_be<T: BinRead>(&mut self) -> BinResult<T>
    where
        T::Args: Default,
    {
        self.read_type(Endian::Big)
    }

    /// Read the given type from the reader with little endian byte order
    fn read_le<T: BinRead>(&mut self) -> BinResult<T>
    where
        T::Args: Default,
    {
        self.read_type(Endian::Little)
    }

    /// Read the given type from the reader with the native byte order
    fn read_ne<T: BinRead>(&mut self) -> BinResult<T>
    where
        T::Args: Default,
    {
        self.read_type(Endian::Native)
    }

    /// Read the given type from the reader using the given endianness.
    fn read_type_args<T: BinRead>(&mut self, endian: Endian, args: T::Args) -> BinResult<T> {
        let options = ReadOptions {
            endian,
            ..Default::default()
        };

        let mut res = T::read_options(self, &options, args.clone())?;
        res.after_parse(self, &options, args)?;

        Ok(res)
    }

    /// Read the given type from the reader with big endian byte order and
    /// arguments
    fn read_be_args<T: BinRead>(&mut self, args: T::Args) -> BinResult<T> {
        self.read_type_args(Endian::Big, args)
    }

    /// Read the given type from the reader with little endian byte order
    /// and arguments
    fn read_le_args<T: BinRead>(&mut self, args: T::Args) -> BinResult<T> {
        self.read_type_args(Endian::Little, args)
    }

    /// Read the given type from the reader with the native byte order
    /// and arguments
    fn read_ne_args<T: BinRead>(&mut self, args: T::Args) -> BinResult<T> {
        self.read_type_args(Endian::Native, args)
    }
}

impl<R: Read + Seek + Sized> BinReaderExt for R {}
