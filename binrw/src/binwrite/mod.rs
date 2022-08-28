mod impls;

use crate::{
    io::{Seek, Write},
    BinResult, Endian,
};

/// The `BinWrite` trait serialises objects and writes them to streams.
///
/// This trait is usually derived, but can also be manually implemented by
/// writing an appropriate [`Args`] type and [`write_options()`] function.
///
/// [`Args`]: Self::Args
/// [`write_options()`]: Self::write_options
///
/// # Derivable
///
/// This trait can be used with `#[derive]` or `#[binwrite]`. Each field
/// of a derived type must either implement `BinWrite` or be annotated with an
/// attribute containing a [`map`], [`try_map`], or [`write_with`] directive.
///
/// [`map`]: crate::docs::attribute#map
/// [`write_with`]: crate::docs::attribute#custom-parserswriters
/// [`try_map`]: crate::docs::attribute#map
///
/// Using `#[binwrite]` instead of `#[derive]` is required when using
/// [temporary fields].
///
/// [temporary fields]: crate::docs::attribute#temp
pub trait BinWrite {
    /// The type used for the `args` parameter of [`write_args()`] and
    /// [`write_options()`].
    ///
    /// When the given type implements [`Default`], convenience functions like
    /// [`write()`] are enabled. `BinWrite` implementations that don’t
    /// receive any arguments should use the `()` type.
    ///
    /// When `BinWrite` is derived, the [`import`] and [`import_tuple`]
    /// directives define this type.
    ///
    /// [`import`]: crate::docs::attribute#arguments
    /// [`import_tuple`]: crate::docs::attribute#arguments
    /// [`write()`]: Self::write
    /// [`write_args()`]: Self::write_args
    /// [`write_options()`]: Self::write_options
    type Args: Clone;

    /// Write `Self` to the writer using default arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        Self::Args: Default,
    {
        self.write_options(writer, &WriteOptions::default(), Self::Args::default())
    }

    /// Write `Self` to the writer using the given arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_args<W: Write + Seek>(&self, writer: &mut W, args: Self::Args) -> BinResult<()> {
        self.write_options(writer, &WriteOptions::default(), args)
    }

    /// Write `Self` to the writer using the given [`WriteOptions`] and
    /// arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: Self::Args,
    ) -> BinResult<()>;
}

/// Runtime options for
/// [`BinWrite::write_options()`](crate::BinWrite::write_options).
#[derive(Default, Clone, Copy)]
pub struct WriteOptions {
    /// The [byte order](crate::Endian) to use when writing data.
    ///
    /// Note that if a derived type uses one of the
    /// [byte order directives](crate::docs::attribute#byte-order), this option
    /// will be overridden by the directive.
    endian: Endian,
}

impl WriteOptions {
    /// Creates a new `WriteOptions` with the given [endianness](crate::Endian).
    #[must_use]
    pub fn new(endian: Endian) -> Self {
        Self { endian }
    }

    /// The [byte order](crate::Endian) to use when writing data.
    ///
    /// Note that if a derived type uses one of the
    /// [byte order directives](crate::docs::attribute#byte-order), this option
    /// will be overridden by the directive.
    #[must_use]
    pub fn endian(&self) -> Endian {
        self.endian
    }

    /// Creates a copy of this `WriteOptions` using the given
    /// [endianness](crate::Endian).
    #[must_use]
    // Lint: For symmetry with `ReadOptions`.
    #[allow(clippy::unused_self)]
    pub fn with_endian(self, endian: Endian) -> Self {
        Self { endian }
    }
}

/// Extension methods for writing [`BinWrite`] objects directly to a writer.
///
/// # Examples
///
/// ```
/// use binrw::{binwrite, BinWriterExt, io::Cursor, Endian};
///
/// #[binwrite]
/// struct MyStruct(u8, u16, u8);
///
/// let mut writer = Cursor::new(Vec::new());
/// writer.write_be(&MyStruct(1, 0xffff, 2)).unwrap();
/// writer.write_type(&0x1234_u16, Endian::Little).unwrap();
///
/// assert_eq!(writer.into_inner(), [1, 0xff, 0xff, 2, 0x34, 0x12]);
/// ```
pub trait BinWriterExt: Write + Seek + Sized {
    /// Write `T` to the reader with the given byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_type<T: BinWrite>(&mut self, value: &T, endian: Endian) -> BinResult<()>
    where
        T::Args: Default,
    {
        self.write_type_args(value, endian, T::Args::default())
    }

    /// Write `T` from the writer assuming big-endian byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_be<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        T::Args: Default,
    {
        self.write_type(value, Endian::Big)
    }

    /// Write `T` from the writer assuming little-endian byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_le<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        T::Args: Default,
    {
        self.write_type(value, Endian::Little)
    }

    /// Write `T` from the writer assuming native-endian byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_ne<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        T::Args: Default,
    {
        self.write_type(value, Endian::Native)
    }

    /// Write `T` from the writer with the given byte order and arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_type_args<T: BinWrite>(
        &mut self,
        value: &T,
        endian: Endian,
        args: T::Args,
    ) -> BinResult<()> {
        let options = WriteOptions::new(endian);

        T::write_options(value, self, &options, args)?;

        Ok(())
    }

    /// Write `T` from the writer, assuming big-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_be_args<T: BinWrite>(&mut self, value: &T, args: T::Args) -> BinResult<()> {
        self.write_type_args(value, Endian::Big, args)
    }

    /// Write `T` from the writer, assuming little-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_le_args<T: BinWrite>(&mut self, value: &T, args: T::Args) -> BinResult<()> {
        self.write_type_args(value, Endian::Little, args)
    }

    /// Write `T` from the writer, assuming native-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_ne_args<T: BinWrite>(&mut self, value: &T, args: T::Args) -> BinResult<()> {
        self.write_type_args(value, Endian::Native, args)
    }
}

impl<W: Write + Seek + Sized> BinWriterExt for W {}
