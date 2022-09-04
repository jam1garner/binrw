mod impls;
mod options;

use crate::{
    io::{Read, Seek},
    BinResult, Endian,
    __private::Required,
    meta::ReadEndian,
};
pub use impls::VecArgs;
pub use options::ReadOptions;

/// The `BinRead` trait reads data from streams and converts it into objects.
///
/// This trait is usually derived, but can also be manually implemented by
/// writing an appropriate [`Args`] type and [`read_options()`] function.
///
/// [`Args`]: Self::Args
/// [`read_options()`]: Self::read_options
///
/// # Derivable
///
/// This trait can be used with `#[derive]` or `#[binread]`. Each field of a
/// derived type must either implement `BinRead` or be annotated with an
/// attribute containing a [`map`], [`try_map`], or [`parse_with`] directive.
///
/// [`map`]: crate::docs::attribute#map
/// [`parse_with`]: crate::docs::attribute#custom-parserswriters
/// [`try_map`]: crate::docs::attribute#map
///
/// Using `#[binread]` instead of `#[derive]` is required when using
/// [temporary fields].
///
/// [temporary fields]: crate::docs::attribute#temp
pub trait BinRead: Sized + 'static {
    /// The type used for the `args` parameter of [`read_args()`] and
    /// [`read_options()`].
    ///
    /// When the given type implements [`Default`], convenience functions like
    /// [`read()`] are enabled. `BinRead` implementations that don’t receive any
    /// arguments should use the `()` type.
    ///
    /// When `BinRead` is derived, the [`import`] and [`import_tuple`]
    /// directives define this type.
    ///
    /// [`import`]: crate::docs::attribute#arguments
    /// [`import_tuple`]: crate::docs::attribute#arguments
    /// [`read()`]: Self::read
    /// [`read_args()`]: Self::read_args
    /// [`read_options()`]: Self::read_options
    type Args: Clone;

    /// Read `Self` from the reader using default arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        Self: ReadEndian,
        Self::Args: Required,
    {
        Self::read_args(reader, Self::Args::args())
    }

    /// Read `Self` from the reader using default arguments and assuming
    /// big-endian byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_be<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        Self::Args: Required,
    {
        Self::read_be_args(reader, Self::Args::args())
    }

    /// Read `Self` from the reader using default arguments and assuming
    /// little-endian byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_le<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        Self::Args: Required,
    {
        Self::read_le_args(reader, Self::Args::args())
    }

    /// Read `T` from the reader assuming native-endian byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_ne<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        Self::Args: Required,
    {
        Self::read_ne_args(reader, Self::Args::args())
    }

    /// Read `Self` from the reader using the given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_args<R: Read + Seek>(reader: &mut R, args: Self::Args) -> BinResult<Self>
    where
        Self: ReadEndian,
    {
        Self::read_options(reader, &ReadOptions::new(Endian::Little), args)
    }

    /// Read `Self` from the reader, assuming big-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_be_args<R: Read + Seek>(reader: &mut R, args: Self::Args) -> BinResult<Self> {
        Self::read_options(reader, &ReadOptions::new(Endian::Big), args)
    }

    /// Read `Self` from the reader, assuming little-endian byte order, using
    /// the given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_le_args<R: Read + Seek>(reader: &mut R, args: Self::Args) -> BinResult<Self> {
        Self::read_options(reader, &ReadOptions::new(Endian::Little), args)
    }

    /// Read `T` from the reader, assuming native-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_ne_args<R: Read + Seek>(reader: &mut R, args: Self::Args) -> BinResult<Self> {
        Self::read_options(reader, &ReadOptions::new(Endian::NATIVE), args)
    }

    /// Read `Self` from the reader using the given [`ReadOptions`] and
    /// arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self>;

    /// Runs any post-processing steps required to finalize construction of the
    /// object.
    ///
    /// # Errors
    ///
    /// If post-processing fails, an [`Error`](crate::Error) variant will be
    /// returned.
    fn after_parse<R: Read + Seek>(
        &mut self,
        _: &mut R,
        _: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<()> {
        Ok(())
    }
}

/// Extension methods for reading [`BinRead`] objects directly from a reader.
///
/// # Examples
///
/// ```
/// use binrw::{BinReaderExt, Endian, io::Cursor};
///
/// let mut reader = Cursor::new(b"\x07\0\0\0\xCC\0\0\x05");
/// let x: u32 = reader.read_le().unwrap();
/// let y: u16 = reader.read_type(Endian::Little).unwrap();
/// let z = reader.read_be::<u16>().unwrap();
///
/// assert_eq!((x, y, z), (7u32, 0xCCu16, 5u16));
/// ```
pub trait BinReaderExt: Read + Seek + Sized {
    /// Read `T` from the reader with the given byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_type<T: BinRead>(&mut self, endian: Endian) -> BinResult<T>
    where
        T::Args: Required,
    {
        self.read_type_args(endian, T::Args::args())
    }

    /// Read `T` from the reader assuming big-endian byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_be<T: BinRead>(&mut self) -> BinResult<T>
    where
        T::Args: Required,
    {
        self.read_type(Endian::Big)
    }

    /// Read `T` from the reader assuming little-endian byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_le<T: BinRead>(&mut self) -> BinResult<T>
    where
        T::Args: Required,
    {
        self.read_type(Endian::Little)
    }

    /// Read `T` from the reader assuming native-endian byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_ne<T: BinRead>(&mut self) -> BinResult<T>
    where
        T::Args: Required,
    {
        self.read_type(Endian::NATIVE)
    }

    /// Read `T` from the reader with the given byte order and arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    fn read_type_args<T: BinRead>(&mut self, endian: Endian, args: T::Args) -> BinResult<T> {
        let options = ReadOptions::new(endian);

        let mut res = T::read_options(self, &options, args.clone())?;
        res.after_parse(self, &options, args)?;

        Ok(res)
    }

    /// Read `T` from the reader, assuming big-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_be_args<T: BinRead>(&mut self, args: T::Args) -> BinResult<T> {
        self.read_type_args(Endian::Big, args)
    }

    /// Read `T` from the reader, assuming little-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_le_args<T: BinRead>(&mut self, args: T::Args) -> BinResult<T> {
        self.read_type_args(Endian::Little, args)
    }

    /// Read `T` from the reader, assuming native-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_ne_args<T: BinRead>(&mut self, args: T::Args) -> BinResult<T> {
        self.read_type_args(Endian::NATIVE, args)
    }
}

impl<R: Read + Seek + Sized> BinReaderExt for R {}
