//! Type definitions for wrappers which represent a layer of indirection within
//! a file.

use crate::{
    io::{Read, Seek, SeekFrom},
    BinRead, BinResult, Endian, NamedArgs, ReadFrom,
};
use core::{
    fmt,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    },
    ops::{Deref, DerefMut},
};

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

/// A converter for non-[`BinRead`] [`FilePtr`] values.
pub enum FilePtrWith<C> {
    #[doc(hidden)]
    _Phantom(Private<C>),
}
#[doc(hidden)]
pub struct Private<T>(core::marker::PhantomData<T>);

impl<Ptr: BinRead<Args = ()> + IntoSeekFrom, C, T: ReadFrom<C>> ReadFrom<FilePtrWith<C>>
    for FilePtr<Ptr, T>
{
    type Args = FilePtrArgs<<T as ReadFrom<C>>::Args>;

    fn read_from<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<Self> {
        Self::read_with_parser(
            |reader, endian, args| <T as ReadFrom<C>>::read_from(reader, endian, args),
            |_, _, _, _| Ok(()),
            reader,
            endian,
            args,
        )
    }
}

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

impl<Ptr: BinRead<Args = ()> + IntoSeekFrom, Value: BinRead> BinRead for FilePtr<Ptr, Value> {
    type Args = FilePtrArgs<Value::Args>;

    /// Reads the offset of the value from the reader.
    ///
    /// The actual value will not be read until
    /// [`after_parse()`](Self::after_parse) is called.
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _: Self::Args,
    ) -> BinResult<Self> {
        Ok(FilePtr {
            ptr: Ptr::read_options(reader, endian, ())?,
            value: None,
        })
    }

    /// Finalizes the `FilePtr` by seeking to and reading the pointed-to value.
    fn after_parse<R>(
        &mut self,
        reader: &mut R,
        endian: Endian,
        args: FilePtrArgs<Value::Args>,
    ) -> BinResult<()>
    where
        R: Read + Seek,
    {
        self.after_parse_with_parser(
            Value::read_options,
            Value::after_parse,
            reader,
            endian,
            args,
        )
    }
}

impl<Ptr: BinRead<Args = ()> + IntoSeekFrom, Value> FilePtr<Ptr, Value> {
    fn read_with_parser<R, Parser, AfterParse, Args>(
        parser: Parser,
        after_parse: AfterParse,
        reader: &mut R,
        endian: Endian,
        args: FilePtrArgs<Args>,
    ) -> BinResult<Self>
    where
        R: Read + Seek,
        Args: Clone,
        Parser: Fn(&mut R, Endian, Args) -> BinResult<Value>,
        AfterParse: Fn(&mut Value, &mut R, Endian, Args) -> BinResult<()>,
    {
        let mut file_ptr = Self {
            ptr: Ptr::read_options(reader, endian, ())?,
            value: None,
        };
        file_ptr.after_parse_with_parser(parser, after_parse, reader, endian, args)?;
        Ok(file_ptr)
    }

    fn after_parse_with_parser<R, Parser, AfterParse, Args>(
        &mut self,
        parser: Parser,
        after_parse: AfterParse,
        reader: &mut R,
        endian: Endian,
        args: FilePtrArgs<Args>,
    ) -> BinResult<()>
    where
        R: Read + Seek,
        Args: Clone,
        Parser: Fn(&mut R, Endian, Args) -> BinResult<Value>,
        AfterParse: Fn(&mut Value, &mut R, Endian, Args) -> BinResult<()>,
    {
        let relative_to = args.offset;
        let before = reader.stream_position()?;
        reader.seek(SeekFrom::Start(relative_to))?;
        reader.seek(self.ptr.into_seek_from())?;

        let mut inner: Value = parser(reader, endian, args.inner.clone())?;

        after_parse(&mut inner, reader, endian, args.inner)?;
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
    #[binrw::parser(reader, endian)]
    pub fn parse<Args>(args: FilePtrArgs<Args>, ...) -> BinResult<Value>
    where
        Args: Clone,
        Value: BinRead<Args = Args>,
    {
        Ok(Self::read_with_parser(
            Value::read_options,
            Value::after_parse,
            reader,
            endian,
            args,
        )?
        .into_inner())
    }

    /// Custom parser for use with the
    /// [`parse_with`](crate::docs::attribute#custom-parserswriters) directive that reads and then
    /// immediately finalizes a [`FilePtr`] using the specified parser, returning the pointed-to
    /// value as the result.
    ///
    /// # Errors
    ///
    /// If reading fails, an [`Error`](crate::Error) variant will be returned.
    pub fn parse_with<R, F, Args>(
        parser: F,
    ) -> impl Fn(&mut R, Endian, FilePtrArgs<Args>) -> BinResult<Value>
    where
        R: Read + Seek,
        Args: Clone,
        F: Fn(&mut R, Endian, Args) -> BinResult<Value>,
    {
        move |reader, endian, args| {
            let after_parse = |_: &mut Value, _: &mut R, _: Endian, _: Args| Ok(());
            Ok(Self::read_with_parser(&parser, after_parse, reader, endian, args)?.into_inner())
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
    pub fn with<R, F, Args>(
        parser: F,
    ) -> impl Fn(&mut R, Endian, FilePtrArgs<Args>) -> BinResult<Self>
    where
        R: Read + Seek,
        Args: Clone,
        F: Fn(&mut R, Endian, Args) -> BinResult<Value>,
    {
        move |reader, endian, args| {
            let after_parse = |_: &mut Value, _: &mut R, _: Endian, _: Args| Ok(());
            Self::read_with_parser(&parser, after_parse, reader, endian, args)
        }
    }

    /// Consumes this object, returning the pointed-to value.
    ///
    /// # Panics
    ///
    /// Will panic if `FilePtr` hasn’t been finalized by calling
    /// [`after_parse()`](Self::after_parse).
    pub fn into_inner(self) -> Value {
        self.value.unwrap()
    }
}

/// Dereferences the value.
///
/// # Panics
///
/// Will panic if `FilePtr` hasn’t been finalized by calling
/// [`after_parse()`](Self::after_parse).
impl<Ptr: IntoSeekFrom, Value> Deref for FilePtr<Ptr, Value> {
    type Target = Value;

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
impl<Ptr: IntoSeekFrom, Value> DerefMut for FilePtr<Ptr, Value> {
    fn deref_mut(&mut self) -> &mut Value {
        match self.value.as_mut() {
            Some(x) => x,
            None => panic!(
                "Deref'd FilePtr before reading (make sure to use FilePtr::after_parse first)"
            ),
        }
    }
}

impl<Ptr, Value> fmt::Debug for FilePtr<Ptr, Value>
where
    Ptr: fmt::Debug + IntoSeekFrom,
    Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref value) = self.value {
            fmt::Debug::fmt(value, f)
        } else {
            f.debug_tuple("UnreadPointer").field(&self.ptr).finish()
        }
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
