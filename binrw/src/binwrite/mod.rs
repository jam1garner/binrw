mod impls;

use crate::{
    io::{Seek, Write},
    BinResult, Endian,
    __private::Required,
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
    #[inline]
    fn write<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        Self: crate::meta::WriteEndian,
        Self::Args: Required,
    {
        self.write_args(writer, Self::Args::args())
    }

    /// Write `Self` to the writer assuming big-endian byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn write_be<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        Self::Args: Required,
    {
        self.write_be_args(writer, Self::Args::args())
    }

    /// Write `Self` to the writer assuming little-endian byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn write_le<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        Self::Args: Required,
    {
        self.write_le_args(writer, Self::Args::args())
    }

    /// Write `Self` to the writer using the given arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn write_args<W: Write + Seek>(&self, writer: &mut W, args: Self::Args) -> BinResult<()>
    where
        Self: crate::meta::WriteEndian,
    {
        self.write_options(writer, Endian::Little, args)
    }

    /// Write `Self` to the writer, assuming big-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn write_be_args<W: Write + Seek>(&self, writer: &mut W, args: Self::Args) -> BinResult<()> {
        self.write_options(writer, Endian::Big, args)
    }

    /// Write `Self` to the writer, assuming little-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn write_le_args<W: Write + Seek>(&self, writer: &mut W, args: Self::Args) -> BinResult<()> {
        self.write_options(writer, Endian::Little, args)
    }

    /// Write `Self` to the writer using the given [`Endian`] and
    /// arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()>;
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
    /// Write `T` to the writer with the given byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_type<T: BinWrite>(&mut self, value: &T, endian: Endian) -> BinResult<()>
    where
        T::Args: Required,
    {
        self.write_type_args(value, endian, T::Args::args())
    }

    /// Write `T` to the writer assuming big-endian byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_be<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        T::Args: Required,
    {
        self.write_type(value, Endian::Big)
    }

    /// Write `T` to the writer assuming little-endian byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_le<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        T::Args: Required,
    {
        self.write_type(value, Endian::Little)
    }

    /// Write `T` to the writer assuming native-endian byte order.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_ne<T: BinWrite>(&mut self, value: &T) -> BinResult<()>
    where
        T::Args: Required,
    {
        self.write_type(value, Endian::NATIVE)
    }

    /// Write `T` to the writer with the given byte order and arguments.
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
        T::write_options(value, self, endian, args)?;

        Ok(())
    }

    /// Write `T` to the writer, assuming big-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_be_args<T: BinWrite>(&mut self, value: &T, args: T::Args) -> BinResult<()> {
        self.write_type_args(value, Endian::Big, args)
    }

    /// Write `T` to the writer, assuming little-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_le_args<T: BinWrite>(&mut self, value: &T, args: T::Args) -> BinResult<()> {
        self.write_type_args(value, Endian::Little, args)
    }

    /// Write `T` to the writer, assuming native-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If writing fails, an [`Error`](crate::Error) variant will be returned.
    fn write_ne_args<T: BinWrite>(&mut self, value: &T, args: T::Args) -> BinResult<()> {
        self.write_type_args(value, Endian::NATIVE, args)
    }
}

impl<W: Write + Seek + Sized> BinWriterExt for W {}
