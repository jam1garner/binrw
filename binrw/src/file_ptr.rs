//! Type definitions for wrappers which represent a layer of indirection within
//! a file.

use core::fmt;
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};
use core::ops::{Deref, DerefMut};

use crate::{
    io::{Read, Seek, SeekFrom},
    BinRead, BinResult, ReadOptions,
};

/// A wrapper type which represents a layer of indirection within a file.
///
/// `FilePtr<P, T>` is composed of two types. The pointer type `P` is the
/// absolute offset to a value within the data, and the value type `T` is
/// the actual pointed-to value. Once a `FilePtr` has been
/// [finalized](crate::BinRead::after_parse), [dereferencing] it will yield the
/// pointed-to value.
///
/// When deriving `BinRead`, [offset](crate::docs::attribute#offset) directives
/// can be used to adjust the offset before the pointed-to value is read.
///
/// [dereferencing]: core::ops::Deref
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
pub struct FilePtr<Ptr: IntoSeekFrom, T> {
    /// The raw offset to the value.
    pub ptr: Ptr,

    /// The pointed-to value.
    pub value: Option<T>,
}

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
/// A type alias for [`FilePtr`] with non-zero  32-bit offsets.
pub type NonZeroFilePtr32<T> = FilePtr<NonZeroU32, T>;
/// A type alias for [`FilePtr`] with non-zero  64-bit offsets.
pub type NonZeroFilePtr64<T> = FilePtr<NonZeroU64, T>;
/// A type alias for [`FilePtr`] with non-zero  128-bit offsets.
pub type NonZeroFilePtr128<T> = FilePtr<NonZeroU128, T>;

impl<Ptr: BinRead<Args = ()> + IntoSeekFrom, BR: BinRead> BinRead for FilePtr<Ptr, BR> {
    type Args = BR::Args;

    /// Reads the offset of the value from the reader.
    ///
    /// The actual value will not be read until
    /// [`after_parse()`](Self::after_parse) is called.
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        Ok(FilePtr {
            ptr: Ptr::read_options(reader, options, ())?,
            value: None,
        })
    }

    /// Finalizes the `FilePtr` by seeking to and reading the pointed-to value.
    fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: BR::Args) -> BinResult<()>
    where
        R: Read + Seek,
    {
        self.after_parse_with_parser(BR::read_options, BR::after_parse, reader, ro, args)
    }
}

impl<Ptr: BinRead<Args = ()> + IntoSeekFrom, T> FilePtr<Ptr, T> {
    fn read_with_parser<R, Parser, AfterParse, Args>(
        parser: Parser,
        after_parse: AfterParse,
        reader: &mut R,
        options: &ReadOptions,
        args: Args,
    ) -> BinResult<Self>
    where
        R: Read + Seek,
        Args: Clone,
        Parser: Fn(&mut R, &ReadOptions, Args) -> BinResult<T>,
        AfterParse: Fn(&mut T, &mut R, &ReadOptions, Args) -> BinResult<()>,
    {
        let mut file_ptr = Self {
            ptr: Ptr::read_options(reader, options, ())?,
            value: None,
        };
        file_ptr.after_parse_with_parser(parser, after_parse, reader, options, args)?;
        Ok(file_ptr)
    }

    fn after_parse_with_parser<R, Parser, AfterParse, Args>(
        &mut self,
        parser: Parser,
        after_parse: AfterParse,
        reader: &mut R,
        options: &ReadOptions,
        args: Args,
    ) -> BinResult<()>
    where
        R: Read + Seek,
        Args: Clone,
        Parser: Fn(&mut R, &ReadOptions, Args) -> BinResult<T>,
        AfterParse: Fn(&mut T, &mut R, &ReadOptions, Args) -> BinResult<()>,
    {
        let relative_to = options.offset();
        let before = reader.stream_position()?;
        reader.seek(SeekFrom::Start(relative_to))?;
        reader.seek(self.ptr.into_seek_from())?;

        let mut inner: T = parser(reader, options, args.clone())?;

        after_parse(&mut inner, reader, options, args)?;
        reader.seek(SeekFrom::Start(before))?;

        self.value = Some(inner);
        Ok(())
    }

    /// Custom parser for use with the
    /// [`parse_with`](crate::docs::attribute#custom-parserswriters) directive that reads
    /// and then immediately finalizes a [`FilePtr`], returning the pointed-to
    /// value as the result.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    pub fn parse<R, Args>(reader: &mut R, options: &ReadOptions, args: Args) -> BinResult<T>
    where
        R: Read + Seek,
        Args: Clone,
        T: BinRead<Args = Args>,
    {
        Ok(
            Self::read_with_parser(T::read_options, T::after_parse, reader, options, args)?
                .into_inner(),
        )
    }

    /// Custom parser for use with the
    /// [`parse_with`](crate::docs::attribute#custom-parserswriters) directive that reads and then
    /// immediately finalizes a [`FilePtr`] using the specified parser, returning the pointed-to
    /// value as the result.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    pub fn parse_with<R, F, Args>(parser: F) -> impl Fn(&mut R, &ReadOptions, Args) -> BinResult<T>
    where
        R: Read + Seek,
        Args: Clone,
        F: Fn(&mut R, &ReadOptions, Args) -> BinResult<T>,
    {
        move |reader, ro, args| {
            let after_parse = |_: &mut T, _: &mut R, _: &ReadOptions, _: Args| Ok(());
            Ok(Self::read_with_parser(&parser, after_parse, reader, ro, args)?.into_inner())
        }
    }

    /// Custom parser for use with the
    /// [`parse_with`](crate::docs::attribute#custom-parserswriters) directive that reads and then
    /// immediately finalizes a [`FilePtr`] using the specified parser, returning the [`FilePtr`]
    /// as the result.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    pub fn with<R, F, Args>(parser: F) -> impl Fn(&mut R, &ReadOptions, Args) -> BinResult<Self>
    where
        R: Read + Seek,
        Args: Clone,
        F: Fn(&mut R, &ReadOptions, Args) -> BinResult<T>,
    {
        move |reader, ro, args| {
            let after_parse = |_: &mut T, _: &mut R, _: &ReadOptions, _: Args| Ok(());
            Self::read_with_parser(&parser, after_parse, reader, ro, args)
        }
    }

    /// Consumes this object, returning the pointed-to value.
    ///
    /// # Panics
    ///
    /// Will panic if `FilePtr` hasn’t been finalized by calling
    /// [`after_parse()`](Self::after_parse).
    pub fn into_inner(self) -> T {
        self.value.unwrap()
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

/// Dereferences the value.
///
/// # Panics
///
/// Will panic if `FilePtr` hasn’t been finalized by calling
/// [`after_parse()`](Self::after_parse).
impl<Ptr: IntoSeekFrom, BR: BinRead> Deref for FilePtr<Ptr, BR> {
    type Target = BR;

    fn deref(&self) -> &Self::Target {
        match self.value.as_ref() {
            Some(x) => x,
            None => panic!(
                "Deref'd FilePtr before reading (make sure to use FilePtr::after_parse first)"
            ),
        }
    }
}

/// # Panics
/// Will panic if the `FilePtr` has not been read yet using
/// [`BinRead::after_parse`](BinRead::after_parse)
impl<Ptr: IntoSeekFrom, BR: BinRead> DerefMut for FilePtr<Ptr, BR> {
    fn deref_mut(&mut self) -> &mut BR {
        match self.value.as_mut() {
            Some(x) => x,
            None => panic!(
                "Deref'd FilePtr before reading (make sure to use FilePtr::after_parse first)"
            ),
        }
    }
}

impl<Ptr, BR> fmt::Debug for FilePtr<Ptr, BR>
where
    Ptr: BinRead<Args = ()> + IntoSeekFrom,
    BR: BinRead + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref value) = self.value {
            fmt::Debug::fmt(value, f)
        } else {
            f.debug_tuple("UnreadPointer").field(&self.ptr).finish()
        }
    }
}

impl<Ptr, BR> PartialEq<FilePtr<Ptr, BR>> for FilePtr<Ptr, BR>
where
    Ptr: BinRead<Args = ()> + IntoSeekFrom,
    BR: BinRead + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
