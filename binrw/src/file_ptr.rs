//! Types definitions and helpers for handling indirection within a file.

use crate::NamedArgs;
use crate::{
    io::{Read, Seek, SeekFrom},
    BinRead, BinResult, Endian,
};
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};
use core::ops::{Deref, DerefMut};

/// A type alias for [`FilePtr`] with 8-bit offsets.
pub type FilePtr8<T> = FilePtr<u8, T>;
/// A type alias for [`FilePtr`] with 16-bit offsets.
pub type FilePtr16<T> = FilePtr<u16, T>;
/// A type alias for [`FilePtr`] with 32-bit offsets.
pub type FilePtr32<T> = FilePtr<u32, T>;
/// A type alias for [`FilePtr`] with 64-bit offsets.
pub type FilePtr64<T> = FilePtr<u64, T>;
/// A type alias for [`FilePtr`] with 128-bit offsets.
pub type FilePtr128<T> = FilePtr<u128, T>;

/// A type alias for [`FilePtr`] with non-zero 8-bit offsets.
pub type NonZeroFilePtr8<T> = FilePtr<NonZeroU8, T>;
/// A type alias for [`FilePtr`] with non-zero 16-bit offsets.
pub type NonZeroFilePtr16<T> = FilePtr<NonZeroU16, T>;
/// A type alias for [`FilePtr`] with non-zero 32-bit offsets.
pub type NonZeroFilePtr32<T> = FilePtr<NonZeroU32, T>;
/// A type alias for [`FilePtr`] with non-zero 64-bit offsets.
pub type NonZeroFilePtr64<T> = FilePtr<NonZeroU64, T>;
/// A type alias for [`FilePtr`] with non-zero 128-bit offsets.
pub type NonZeroFilePtr128<T> = FilePtr<NonZeroU128, T>;

/// A wrapper type which represents a layer of indirection within a file.
///
/// The pointer type `Ptr` is an offset to a value within the data stream, and
/// the value type `T` is the value at that offset.
///
/// [Dereferencing] a `FilePtr` yields the pointed-to value.
///
/// When deriving `BinRead`, the [offset](crate::docs::attribute#offset)
/// directive can be used to adjust the offset before the pointed-to value is
/// read.
///
/// [Dereferencing]: core::ops::Deref
///
/// # Performance
///
/// Using `FilePtr` directly is not efficient when reading offset tables because
/// it immediately seeks to read the pointed-to value. Instead:
///
/// 1. Read the offset table as a `Vec<{integer}>`, then use
///   [`parse_from_iter`] to create a collection of `Vec<T>`.
///
/// 2. Read the offset table as a `Vec<{integer}>`, then add a function
///    to your type that lazily reads the pointed-to value:
///
/// ```
/// # use binrw::{args, BinRead, BinResult, BinReaderExt, helpers::until_eof, io::{Cursor, Read, Seek, SeekFrom}};
/// #[derive(BinRead)]
/// #[br(big)]
/// struct Header {
///     count: u16,
///
///     #[br(args { count: count.into() })]
///     offsets: Vec<u16>,
/// }
///
/// #[derive(BinRead)]
/// # #[derive(Debug, Eq, PartialEq)]
/// #[br(big)]
/// struct Item(u8);
///
/// #[derive(BinRead)]
/// #[br(big, stream = s)]
/// struct Object {
///     header: Header,
///     #[br(try_calc = s.stream_position())]
///     data_offset: u64,
/// }
///
/// impl Object {
///     pub fn get<R: Read + Seek>(&self, source: &mut R, index: usize) -> Option<BinResult<Item>> {
///         self.header.offsets.get(index).map(|offset| {
///             let offset = self.data_offset + u64::from(*offset);
///             source.seek(SeekFrom::Start(offset))?;
///             Item::read(source)
///         })
///     }
/// }
///
/// # let mut s = Cursor::new(b"\0\x02\0\x01\0\0\x03\x04");
/// # let x = Object::read(&mut s).unwrap();
/// # assert!(matches!(x.get(&mut s, 0), Some(Ok(Item(4)))));
/// # assert!(matches!(x.get(&mut s, 1), Some(Ok(Item(3)))));
/// # assert!(matches!(x.get(&mut s, 2), None));
/// ```
///
/// # Examples
///
/// ```
/// # use binrw::{prelude::*, io::Cursor, FilePtr};
/// #
/// #[derive(BinRead)]
/// struct Test {
///     indirect_value: FilePtr<u32, u8>
/// }
///
/// let test: Test = Cursor::new(b"\0\0\0\x08\0\0\0\0\xff").read_be().unwrap();
/// assert_eq!(test.indirect_value.ptr, 8);
/// assert_eq!(*test.indirect_value, 0xFF);
/// ```
///
/// Example data mapped out:
///
/// ```hex
///           [pointer]           [value]
/// 00000000: 0000 0008 0000 0000 ff                   ............
/// ```
#[derive(Debug, Eq)]
pub struct FilePtr<Ptr: IntoSeekFrom, T> {
    /// The raw offset to the value.
    pub ptr: Ptr,

    /// The pointed-to value.
    pub value: T,
}

impl<Ptr, Value> BinRead for FilePtr<Ptr, Value>
where
    Ptr: for<'a> BinRead<Args<'a> = ()> + IntoSeekFrom,
    Value: BinRead,
{
    type Args<'a> = FilePtrArgs<Value::Args<'a>>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let ptr = Ptr::read_options(reader, endian, ())?;
        let value = Self::read_value(ptr, Value::read_options, reader, endian, args)?;
        Ok(FilePtr { ptr, value })
    }
}

impl<Ptr, Value> FilePtr<Ptr, Value>
where
    Ptr: IntoSeekFrom,
{
    /// Reads an offset, then seeks to and parses the pointed-to value using the
    /// [`BinRead`] implementation for `Value`. Returns the pointed-to value.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[binrw::parser(reader, endian)]
    pub fn parse<Args>(args: FilePtrArgs<Args>, ...) -> BinResult<Value>
    where
        Ptr: for<'a> BinRead<Args<'a> = ()> + IntoSeekFrom,
        Value: for<'a> BinRead<Args<'a> = Args>,
    {
        Self::read_options(reader, endian, args).map(Self::into_inner)
    }

    /// Creates a parser that reads an offset, then seeks to and parses the
    /// pointed-to value using the given `parser` function. Returns the
    /// pointed-to value.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    pub fn parse_with<R, F, Args>(
        parser: F,
    ) -> impl Fn(&mut R, Endian, FilePtrArgs<Args>) -> BinResult<Value>
    where
        R: Read + Seek,
        F: Fn(&mut R, Endian, Args) -> BinResult<Value>,
        Ptr: for<'a> BinRead<Args<'a> = ()> + IntoSeekFrom,
    {
        let parser = Self::with(parser);
        move |reader, endian, args| parser(reader, endian, args).map(Self::into_inner)
    }

    /// Creates a parser that reads an offset, then seeks to and parses the
    /// pointed-to value using the given `parser` function. Returns a
    /// [`FilePtr`] containing the offset and value.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    pub fn with<R, F, Args>(
        parser: F,
    ) -> impl Fn(&mut R, Endian, FilePtrArgs<Args>) -> BinResult<Self>
    where
        R: Read + Seek,
        F: Fn(&mut R, Endian, Args) -> BinResult<Value>,
        Ptr: for<'a> BinRead<Args<'a> = ()> + IntoSeekFrom,
    {
        move |reader, endian, args| {
            let ptr = Ptr::read_options(reader, endian, ())?;
            let value = Self::read_value(ptr, &parser, reader, endian, args)?;
            Ok(Self { ptr, value })
        }
    }

    /// Consumes this object, returning the pointed-to value.
    pub fn into_inner(self) -> Value {
        self.value
    }

    fn read_value<R, Parser, Args>(
        ptr: Ptr,
        parser: Parser,
        reader: &mut R,
        endian: Endian,
        args: FilePtrArgs<Args>,
    ) -> BinResult<Value>
    where
        R: Read + Seek,
        Parser: FnOnce(&mut R, Endian, Args) -> BinResult<Value>,
    {
        let relative_to = args.offset;
        let before = reader.stream_position()?;
        reader.seek(SeekFrom::Start(relative_to))?;
        reader.seek(ptr.into_seek_from())?;
        let value = parser(reader, endian, args.inner);
        reader.seek(SeekFrom::Start(before))?;
        value
    }
}

impl<Ptr: IntoSeekFrom, Value: BinRead> Deref for FilePtr<Ptr, Value> {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<Ptr: IntoSeekFrom, Value: BinRead> DerefMut for FilePtr<Ptr, Value> {
    fn deref_mut(&mut self) -> &mut Value {
        &mut self.value
    }
}

impl<Ptr, Value> PartialEq<FilePtr<Ptr, Value>> for FilePtr<Ptr, Value>
where
    Ptr: PartialEq + IntoSeekFrom,
    Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// Creates a parser that reads a collection of values from an iterator of
/// file offsets using the [`BinRead`] implementation of `Value`.
///
/// Offsets are treated as relative to the position of the reader when
/// parsing begins. Use the [`seek_before`] directive to reposition the
/// stream in this case.
///
/// [`seek_before`]: crate::docs::attribute#padding-and-alignment
///
/// # Examples
///
/// ```
/// # use binrw::{args, BinRead, BinReaderExt, io::Cursor};
/// #[derive(BinRead)]
/// #[br(big)]
/// struct Header {
///     count: u16,
///
///     #[br(args { count: count.into() })]
///     offsets: Vec<u16>,
/// }
///
/// #[derive(BinRead)]
/// #[br(big)]
/// struct Object {
///     header: Header,
///     #[br(parse_with = binrw::file_ptr::parse_from_iter(header.offsets.iter().copied()))]
///     values: Vec<u8>,
/// }
///
/// # let mut x = Cursor::new(b"\0\x02\0\x01\0\0\x03\x04");
/// # let x = Object::read(&mut x).unwrap();
/// # assert_eq!(x.values, &[4, 3]);
/// ```
pub fn parse_from_iter<Ptr, Value, Ret, Args, It, Reader>(
    it: It,
) -> impl FnOnce(&mut Reader, Endian, Args) -> BinResult<Ret>
where
    Ptr: IntoSeekFrom,
    Value: for<'a> BinRead<Args<'a> = Args>,
    Ret: FromIterator<Value>,
    Args: Clone,
    It: IntoIterator<Item = Ptr>,
    Reader: Read + Seek,
{
    parse_from_iter_with(it, Value::read_options)
}

/// Creates a parser that reads a collection of values from an iterator of
/// file offsets using the given `parser` function.
///
/// Offsets are treated as relative to the position of the reader when
/// parsing begins. Use the [`seek_before`] directive to reposition the
/// stream in this case.
///
/// [`seek_before`]: crate::docs::attribute#padding-and-alignment
///
/// # Examples
///
/// ```
/// # use binrw::{args, BinRead, BinReaderExt, io::Cursor};
/// #[derive(BinRead)]
/// #[br(big)]
/// struct Header {
///     count: u16,
///
///     #[br(args { count: count.into() })]
///     offsets: Vec<u16>,
/// }
///
/// # #[derive(Debug, Eq, PartialEq)]
/// struct Item(u8);
///
/// #[derive(BinRead)]
/// #[br(big)]
/// struct Object {
///     header: Header,
///     #[br(parse_with = binrw::file_ptr::parse_from_iter_with(header.offsets.iter().copied(), |reader, endian, args| {
///        u8::read_options(reader, endian, args).map(Item)
///     }))]
///     values: Vec<Item>,
/// }
///
/// # let mut x = Cursor::new(b"\0\x02\0\x01\0\0\x03\x04");
/// # let x = Object::read(&mut x).unwrap();
/// # assert_eq!(x.values, &[Item(4), Item(3)]);
/// ```
pub fn parse_from_iter_with<Ptr, Value, Ret, Args, It, F, Reader>(
    it: It,
    parser: F,
) -> impl FnOnce(&mut Reader, Endian, Args) -> BinResult<Ret>
where
    Ptr: IntoSeekFrom,
    Ret: FromIterator<Value>,
    Args: Clone,
    It: IntoIterator<Item = Ptr>,
    F: Fn(&mut Reader, Endian, Args) -> BinResult<Value>,
    Reader: Read + Seek,
{
    move |reader, endian, args| {
        let base_pos = reader.stream_position()?;
        it.into_iter()
            .map(move |ptr| {
                // Avoid unnecessary seeks:
                // 1. Unnecessary seeking backwards to the base position
                //    will cause forward-only readers to fail always even if
                //    the offsets are ordered;
                // 2. Seeks that change the position when it does not need
                //    to change may unnecessarily flush a buffered reader
                //    cache.
                match ptr.into_seek_from() {
                    seek @ SeekFrom::Current(offset) => {
                        if let Some(new_pos) = base_pos.checked_add_signed(offset) {
                            if new_pos != reader.stream_position()? {
                                reader.seek(SeekFrom::Start(new_pos))?;
                            }
                        } else {
                            reader.seek(seek)?;
                        }
                    }
                    seek => {
                        reader.seek(seek)?;
                    }
                }

                parser(reader, endian, args.clone())
            })
            .collect()
    }
}

/// A trait to convert from an integer into
/// [`SeekFrom::Current`](crate::io::SeekFrom::Current).
pub trait IntoSeekFrom: Copy {
    /// Converts the value.
    fn into_seek_from(self) -> SeekFrom;
}

macro_rules! impl_into_seek_from {
    ($($t:ty),*) => {
        $(
            impl IntoSeekFrom for $t {
                fn into_seek_from(self) -> SeekFrom {
                    SeekFrom::Current(TryInto::try_into(self).unwrap())
                }
            }
        )*
    };
}

impl_into_seek_from!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

macro_rules! impl_into_seek_from_for_non_zero {
    ($($t:ty),*) => {
        $(
            impl IntoSeekFrom for $t {
                fn into_seek_from(self) -> SeekFrom {
                    self.get().into_seek_from()
                }
            }
        )*
    };
}

impl_into_seek_from_for_non_zero!(
    NonZeroI128,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI8,
    NonZeroU128,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU8
);

/// Named arguments for the [`BinRead::read_options()`] implementation of [`FilePtr`].
///
/// The `inner` field can be omitted completely if the inner type doesn’t
/// require arguments, in which case a default value will be used.
#[derive(Clone, Default, NamedArgs)]
pub struct FilePtrArgs<Inner> {
    /// An absolute offset added to the [`FilePtr::ptr`](crate::FilePtr::ptr)
    /// offset before reading the pointed-to value.
    #[named_args(default = 0)]
    pub offset: u64,

    /// The [arguments](crate::BinRead::Args) for the inner type.
    #[named_args(try_optional)]
    pub inner: Inner,
}
