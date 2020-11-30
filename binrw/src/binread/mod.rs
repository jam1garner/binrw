use core::any::{Any, TypeId};
use crate::io::{Read, Seek};
use crate::{BinResult, Endian};

mod impls;
mod options;

pub use options::ReadOptions;

/// A `BinRead` trait allows reading a structure from anything that implements [`io::Read`](io::Read) and [`io::Seek`](io::Seek)
/// BinRead is implemented on the type to be read out of the given reader
pub trait BinRead: Sized {
    /// The type of arguments needed to be supplied in order to read this type, usually a tuple.
    ///
    /// **NOTE:** For types that don't require any arguments, use the unit (`()`) type. This will allow [`read`](BinRead::read) to be used.
    type Args: Any + Copy;

    /// Read the type from the reader while assuming no arguments have been passed
    ///
    /// # Panics
    /// Panics if there is no [`args_default`](BinRead::args_default) implementation
    fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        let args = match Self::args_default() {
            Some(args) => args,
            None => panic!("Must pass args, no args_default implemented")
        };

        Self::read_options(reader, &ReadOptions::default(), args)
    }

    /// Read the type from the reader using the specified arguments
    fn read_args<R: Read + Seek>(reader: &mut R, args: Self::Args) -> BinResult<Self> {
        Self::read_options(reader, &ReadOptions::default(), args)
    }

    /// Read the type from the reader
    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self>;

    fn after_parse<R: Read + Seek>(&mut self, _: &mut R, _: &ReadOptions, _: Self::Args) -> BinResult<()> {
        Ok(())
    }

    /// The default arguments to be used when using the [`read`](BinRead::read) shortcut method.
    /// Override this for any type that optionally requries arguments
    fn args_default() -> Option<Self::Args> {
        // Trick to effectively get specialization on stable, should constant-folded away
        // Returns `Some(())` if Self::Args == (), otherwise returns `None`
        if TypeId::of::<Self::Args>() == TypeId::of::<()>() {
            Some(unsafe{
                core::mem::MaybeUninit::uninit().assume_init()
            })
        } else {
            None
        }
    }
}

/// An extension trait for [`io::Read`](io::Read) to provide methods for reading a value directly
///
/// ## Example
/// ```rust
/// use binrw::prelude::*; // BinReadExt is in the prelude
/// use binrw::endian::LE;
/// use std::io::Cursor;
///
/// fn main() {
///     let mut reader = Cursor::new(b"\x07\0\0\0\xCC\0\0\x05");
///     let x: u32 = reader.read_le().unwrap();
///     let y: u16 = reader.read_type(LE).unwrap();
///     let z = reader.read_be::<u16>().unwrap();
///
///     assert_eq!((x, y, z), (7u32, 0xCCu16, 5u16));
/// }
/// ```
pub trait BinReaderExt: Read + Seek + Sized {
    /// Read the given type from the reader using the given endianness.
    fn read_type<T: BinRead>(&mut self, endian: Endian) -> BinResult<T> {
        let args = match T::args_default() {
            Some(args) => args,
            None => panic!("Must pass args, no args_default implemented")
        };

        let options = ReadOptions{
            endian, ..Default::default()
        };

        let mut res = T::read_options(self, &options, args)?;
        res.after_parse(self, &options, args)?;

        Ok(res)
    }

    /// Read the given type from the reader with big endian byteorder
    fn read_be<T: BinRead>(&mut self) -> BinResult<T> {
        self.read_type(Endian::Big)
    }

    /// Read the given type from the reader with little endian byteorder
    fn read_le<T: BinRead>(&mut self) -> BinResult<T> {
        self.read_type(Endian::Little)
    }

    /// Read the given type from the reader with the native byteorder
    fn read_ne<T: BinRead>(&mut self) -> BinResult<T> {
        self.read_type(Endian::Native)
    }
}

impl<R: Read + Seek + Sized> BinReaderExt for R {}
