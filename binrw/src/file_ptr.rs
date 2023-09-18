//! Type definitions for wrappers which represent a layer of indirection within
//! a file.

use crate::NamedArgs;
use crate::{
    io::{Read, Seek, SeekFrom},
    BinRead, BinResult, Endian,
};
use core::fmt;
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
/// `FilePtr<P, T>` is composed of two types. The pointer type `P` is the
/// absolute offset to a value within the data, and the value type `T` is
/// the actual pointed-to value. [Dereferencing] it will yield the
/// pointed-to value.
///
/// When deriving `BinRead`, the [offset](crate::docs::attribute#offset)
/// directive can be used to adjust the offset before the pointed-to value is
/// read.
///
/// [Dereferencing]: core::ops::Deref
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

    /// Reads the offset of the value from the reader.
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let ptr = Ptr::read_options(reader, endian, ())?;
        let value = Self::after_parse_with_parser(ptr, Value::read_options, reader, endian, args)?;
        Ok(FilePtr { ptr, value })
    }
}

impl<Ptr, Value> FilePtr<Ptr, Value>
where
    Ptr: for<'a> BinRead<Args<'a> = ()> + IntoSeekFrom,
{
    fn after_parse_with_parser<R, Parser, Args>(
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

    /// Custom parser for use with the
    /// [`parse_with`](crate::docs::attribute#custom-parserswriters) directive
    /// that reads a [`FilePtr`] and returns the pointed-to value as the result.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    #[binrw::parser(reader, endian)]
    pub fn parse<Args>(args: FilePtrArgs<Args>, ...) -> BinResult<Value>
    where
        Value: for<'a> BinRead<Args<'a> = Args>,
    {
        Self::read_options(reader, endian, args).map(Self::into_inner)
    }

    /// Custom parser for use with the
    /// [`parse_with`](crate::docs::attribute#custom-parserswriters) directive
    /// that reads a [`FilePtr`] using the specified parser, returning the
    /// pointed-to value as the result.
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
    {
        move |reader, endian, args| {
            let ptr = Ptr::read_options(reader, endian, ())?;
            Self::after_parse_with_parser(ptr, &parser, reader, endian, args)
        }
    }

    /// Custom parser for use with the
    /// [`parse_with`](crate::docs::attribute#custom-parserswriters) directive
    /// that reads a [`FilePtr`] using the specified parser, returning the
    /// [`FilePtr`] as the result.
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
    {
        move |reader, endian, args| {
            let ptr = Ptr::read_options(reader, endian, ())?;
            Self::after_parse_with_parser(ptr, &parser, reader, endian, args)
                .map(|value| Self { ptr, value })
        }
    }

    /// Consumes this object, returning the pointed-to value.
    pub fn into_inner(self) -> Value {
        self.value
    }
}

impl<Ptr, Value> Deref for FilePtr<Ptr, Value>
where
    Ptr: IntoSeekFrom,
{
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<Ptr, Value> DerefMut for FilePtr<Ptr, Value>
where
    Ptr: IntoSeekFrom,
{
    fn deref_mut(&mut self) -> &mut Value {
        &mut self.value
    }
}

impl<Ptr, Value> PartialEq<FilePtr<Ptr, Value>> for FilePtr<Ptr, Value>
where
    Ptr: IntoSeekFrom,
    Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// A trait to convert from an integer into
/// [`SeekFrom::Current`](crate::io::SeekFrom::Current).
pub trait IntoSeekFrom: Copy + fmt::Debug {
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
