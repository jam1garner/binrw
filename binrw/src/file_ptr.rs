//! Type definitions for wrappers which represent a layer of indirection within
//! a file.

use core::fmt;
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
/// When deriving `BinRead`, [offset](crate::attribute#offset) directives
/// can be used to adjust the offset before the pointed-to value is read.
///
/// [dereferencing]: core::ops::Deref
///
/// # Examples
///
/// ```rust
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
pub struct FilePtr<Ptr: IntoSeekFrom, BR: BinRead> {
    /// The raw offset to the value.
    pub ptr: Ptr,

    /// The pointed-to value.
    pub value: Option<BR>,
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
        let relative_to = ro.offset;
        let before = reader.seek(SeekFrom::Current(0))?;
        reader.seek(SeekFrom::Start(relative_to))?;
        reader.seek(self.ptr.into_seek_from())?;

        let mut inner: BR = BinRead::read_options(reader, ro, args.clone())?;

        inner.after_parse(reader, ro, args)?;

        self.value = Some(inner);

        reader.seek(SeekFrom::Start(before))?;
        Ok(())
    }
}

impl<Ptr: BinRead<Args = ()> + IntoSeekFrom, BR: BinRead> FilePtr<Ptr, BR> {
    /// Custom parser for use with the
    /// [`parse_with`](crate::attribute#custom-parsers) directive that reads
    /// and then immediately finalizes a [`FilePtr`], returning the pointed-to
    /// value as the result.
    pub fn parse<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: BR::Args,
    ) -> BinResult<BR> {
        let mut ptr: Self = Self::read_options(reader, options, args.clone())?;
        let saved_pos = reader.seek(SeekFrom::Current(0))?;
        ptr.after_parse(reader, options, args)?;
        reader.seek(SeekFrom::Start(saved_pos))?;
        Ok(ptr.into_inner())
    }

    /// Consumes this object, returning the pointed-to value.
    ///
    /// # Panics
    ///
    /// Will panic if `FilePtr` hasn’t been finalized by calling
    /// [`after_parse()`](Self::after_parse).
    pub fn into_inner(self) -> BR {
        self.value.unwrap()
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
                    SeekFrom::Current(core::convert::TryInto::try_into(self).unwrap())
                }
            }
        )*
    };
}

impl_into_seek_from!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

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

/// ## Panics
/// Will panic if the FilePtr has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
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
            write!(f, "UnreadPointer")
        }
    }
}

impl<Ptr, BR> PartialEq<FilePtr<Ptr, BR>> for FilePtr<Ptr, BR>
where
    Ptr: BinRead<Args = ()> + IntoSeekFrom,
    BR: BinRead + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
