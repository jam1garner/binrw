use crate::{
    io::{Seek, Write},
    BinResult, Endian,
};

mod impls;

/// A trait for writing a given type to a writer
pub trait BinWrite {
    /// The type of arguments needed to be supplied in order to write this type, usually a tuple.
    ///
    /// **Note:** For types that don't require any arguments, use the unit (`()`) type.
    /// This will allow [`write_to`](BinWrite::write_to) to be used.
    type Args: Clone;

    /// Write a type to a writer while assuming no arguments are needed.
    fn write_to<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        Self::Args: Default,
    {
        self.write_options(writer, &WriteOptions::default(), Self::Args::default())
    }

    /// Write the type to a writer while providing the default [`WriteOptions`]
    fn write_with_args<W: Write + Seek>(&self, writer: &mut W, args: Self::Args) -> BinResult<()> {
        self.write_options(writer, &WriteOptions::default(), args)
    }

    /// Write the type to a writer, given the options on how to write it and the type-specific
    /// arguments
    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: Self::Args,
    ) -> BinResult<()>;
}

/// Options for how data should be written
///
/// Functionally the purpose of WriteOptions is simple: maintaining context which is implicitly
/// passed throughout all types being written.
#[derive(Default, Clone)]
pub struct WriteOptions {
    endian: Endian,
}

impl WriteOptions {
    /// Create a new `WriteOptions`. Additional fields can be instantiated using `.with_{field}`.
    pub fn new(endian: Endian) -> Self {
        Self { endian }
    }

    /// Retrieves the specified endian
    pub fn endian(&self) -> Endian {
        self.endian
    }

    /// Returns the same `WriteOptions` but with the endian set
    pub fn with_endian(self, endian: Endian) -> Self {
        WriteOptions { endian }
    }
}

/// Extension methods for writing [`BinWrite`] objects directly to a writer.
///
/// # Examples
///
/// ```rust
/// use binrw::{binwrite, BinWriterExt, io::Cursor, Endian};
///
/// #[binwrite]
/// struct MyStruct(u8, u16, u8);
///
/// let mut writer = Cursor::new(Vec::new());
/// writer.write_be(&MyStruct(1, 0xffff, 2)).unwrap();
/// writer.write_type(&0x1234_u16, Endian::Little).unwrap();
///
/// assert_eq!(&writer.into_inner()[..], &[1, 0xff, 0xff, 2, 0x34, 0x12][..]);
/// ```
pub trait BinWriterExt: Write + Seek + Sized {
    /// Write `T` from the writer with the given byte order.
    fn write_type<T: BinWrite>(&mut self, value: &T, endian: Endian) -> BinResult<()>
    where
        T::Args: Default,
    {
        self.write_type_args(value, endian, T::Args::default())
    }

    /// Write `T` from the writer assuming big-endian byte order.
    fn write_be<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        T::Args: Default,
    {
        self.write_type(value, Endian::Big)
    }

    /// Write `T` from the writer assuming little-endian byte order.
    fn write_le<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        T::Args: Default,
    {
        self.write_type(value, Endian::Little)
    }

    /// Write `T` from the writer assuming native-endian byte order.
    fn write_ne<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        T::Args: Default,
    {
        self.write_type(value, Endian::Native)
    }

    /// Write `T` from the writer with the given byte order and arguments.
    fn write_type_args<T: BinWrite>(
        &mut self,
        value: &T,
        endian: Endian,
        args: T::Args,
    ) -> BinResult<()> {
        let options = WriteOptions::new(endian);

        T::write_options(value, self, &options, args)?;
        //res.after_parse(self, &options, args)?;

        Ok(())
    }

    /// Write `T` from the writer, assuming big-endian byte order, using the
    /// given arguments.
    fn write_be_args<T: BinWrite>(&mut self, value: &T, args: T::Args) -> BinResult<()> {
        self.write_type_args(value, Endian::Big, args)
    }

    /// Write `T` from the writer, assuming little-endian byte order, using the
    /// given arguments.
    fn write_le_args<T: BinWrite>(&mut self, value: &T, args: T::Args) -> BinResult<()> {
        self.write_type_args(value, Endian::Little, args)
    }

    /// Write `T` from the writer, assuming native-endian byte order, using the
    /// given arguments.
    fn write_ne_args<T: BinWrite>(&mut self, value: &T, args: T::Args) -> BinResult<()> {
        self.write_type_args(value, Endian::Native, args)
    }
}

impl<W: Write + Seek + Sized> BinWriterExt for W {}
