mod impls;

use crate::{
    io::{Read, Seek},
    BinResult, Endian,
    __private::Required,
    meta::ReadEndian,
};
pub use impls::VecArgs;

/// The `BinRead` trait reads data from streams and converts it into objects.
///
/// This trait is usually derived, but can also be manually implemented by
/// writing an appropriate [`Args`] type and [`read_options()`] function.
///
/// [`Args`]: Self::Args
/// [`read_options()`]: Self::read_options
///
/// # Examples
///
/// Create a parser for a relative offset with an optional base offset.
///
/// ```
/// # use binrw::{BinRead, BinResult, NamedArgs};
/// # use binrw::io::{Read, Seek, SeekFrom};
/// #[derive(Clone, Default, NamedArgs)]
/// struct CustomPtr32Args<Inner: Default> {
///     #[named_args(default = 0)]
///     offset: u64,
///     #[named_args(default = Default::default())]
///     inner: Inner,
/// }
///
/// struct CustomPtr32<T>(T);
///
/// impl<T, TArgs> BinRead for CustomPtr32<T>
/// where
///     for<'a> T: BinRead<Args<'a> = TArgs>,
///     TArgs: Default
/// {
///     type Args<'a> = CustomPtr32Args<TArgs>;
///
///     fn read_options<R: Read + Seek>(
///         reader: &mut R,
///         endian: binrw::Endian,
///         args: Self::Args<'_>,
///     ) -> BinResult<Self> {
///         let offset = u32::read_options(reader, endian, ())?;
///         let position = reader.stream_position()?;
///
///         reader.seek(SeekFrom::Start(args.offset + offset as u64))?;
///         let value = T::read_options(reader, endian, args.inner)?;
///         reader.seek(SeekFrom::Start(position))?;
///
///         Ok(CustomPtr32(value))
///     }
/// }
/// ```
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
pub trait BinRead: Sized {
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
    type Args<'a>;

    /// Read `Self` from the reader using default arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self>
    where
        Self: ReadEndian,
        for<'a> Self::Args<'a>: Required,
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
        for<'a> Self::Args<'a>: Required,
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
        for<'a> Self::Args<'a>: Required,
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
        for<'a> Self::Args<'a>: Required,
    {
        Self::read_ne_args(reader, Self::Args::args())
    }

    /// Read `Self` from the reader using the given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_args<R: Read + Seek>(reader: &mut R, args: Self::Args<'_>) -> BinResult<Self>
    where
        Self: ReadEndian,
    {
        Self::read_options(reader, Endian::Little, args)
    }

    /// Read `Self` from the reader, assuming big-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_be_args<R: Read + Seek>(reader: &mut R, args: Self::Args<'_>) -> BinResult<Self> {
        Self::read_options(reader, Endian::Big, args)
    }

    /// Read `Self` from the reader, assuming little-endian byte order, using
    /// the given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_le_args<R: Read + Seek>(reader: &mut R, args: Self::Args<'_>) -> BinResult<Self> {
        Self::read_options(reader, Endian::Little, args)
    }

    /// Read `T` from the reader, assuming native-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_ne_args<R: Read + Seek>(reader: &mut R, args: Self::Args<'_>) -> BinResult<Self> {
        Self::read_options(reader, Endian::NATIVE, args)
    }

    /// Read `Self` from the reader using the given [`Endian`] and
    /// arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
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
        _: Endian,
        _: Self::Args<'_>,
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
    fn read_type<'a, T>(&mut self, endian: Endian) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Required + Clone,
    {
        self.read_type_args(endian, T::Args::args())
    }

    /// Read `T` from the reader assuming big-endian byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_be<'a, T>(&mut self) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Required + Clone,
    {
        self.read_type(Endian::Big)
    }

    /// Read `T` from the reader assuming little-endian byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_le<'a, T>(&mut self) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Required + Clone,
    {
        self.read_type(Endian::Little)
    }

    /// Read `T` from the reader assuming native-endian byte order.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_ne<'a, T>(&mut self) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Required + Clone,
    {
        self.read_type(Endian::NATIVE)
    }

    /// Read `T` from the reader with the given byte order and arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    fn read_type_args<'a, T>(&mut self, endian: Endian, args: T::Args<'a>) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Clone,
    {
        let mut res = T::read_options(self, endian, args.clone())?;
        res.after_parse(self, endian, args)?;

        Ok(res)
    }

    /// Read `T` from the reader, assuming big-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_be_args<'a, T>(&mut self, args: T::Args<'a>) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Clone,
    {
        self.read_type_args(Endian::Big, args)
    }

    /// Read `T` from the reader, assuming little-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_le_args<'a, T>(&mut self, args: T::Args<'a>) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Clone,
    {
        self.read_type_args(Endian::Little, args)
    }

    /// Read `T` from the reader, assuming native-endian byte order, using the
    /// given arguments.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[inline]
    fn read_ne_args<'a, T>(&mut self, args: T::Args<'a>) -> BinResult<T>
    where
        T: BinRead,
        T::Args<'a>: Clone,
    {
        self.read_type_args(Endian::NATIVE, args)
    }
}

impl<R: Read + Seek + Sized> BinReaderExt for R {}
