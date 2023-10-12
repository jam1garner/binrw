//! Type definitions and helpers for handling indirection within a file.
//!
//! # Best practices
//!
//! Indirections that are not collections (e.g. a single offset to a global file
//! header) can use `FilePtr` to immediately read the offset and then parse the
//! pointed-to value. However, using `FilePtr` inside a collection is
//! inefficient because it seeks to and reads each pointed-to value immediately
//! after the offset is read. In these cases, it is faster to read the offset
//! table into a collection (e.g. `Vec<u32>`) and then either pass it to
//! [`parse_from_iter`] or write a function that is called to lazily load values
//! as needed.
//!
//! ## Using `parse_from_iter` to read an offset table
//!
//! ### With relative offsets
//!
//! In this example, the offsets in the offset table start counting from the
//! beginning of the values section, and are in a random order.
//!
//! Since the values section exists immediately after the offset table, no
//! seeking is required before reading the values.
//!
//! Since the offsets are in a random order, the position of the stream must be
//! returned to a known state using `restore_position` on the values field.
//! Then, `seek_before` is used on the next field to skip past the values data
//! and continue reading the rest of the object.
//!
//! ```
//! # use binrw::{args, BinRead, BinReaderExt, io::{Cursor, SeekFrom}};
//! use binrw::file_ptr::parse_from_iter;
//!
//! #[derive(BinRead)]
//! #[br(big)]
//! struct Object {
//!     count: u16,
//!     #[br(args { count: count.into() })]
//!     offsets: Vec<u16>,
//!     #[br(parse_with = parse_from_iter(offsets.iter().copied()), restore_position)]
//!     values: Vec<u8>,
//!     #[br(seek_before(SeekFrom::Current(count.into())))]
//!     extra: u16,
//! }
//!
//! # let mut x = Cursor::new(b"\0\x02\0\x01\0\0\x03\x04\xff\xff");
//! # let x = Object::read(&mut x).unwrap();
//! # assert_eq!(x.values, &[4, 3]);
//! # assert_eq!(x.extra, 0xffff);
//! ```
//!
//! ### With absolute offsets
//!
//! In this example, the offsets in the offset table start from the beginning of
//! the file, and are in sequential order.
//!
//! Since the offsets start from the beginning of the file, it is necessary to
//! use `seek_before` to reposition the stream to the beginning of the file
//! before reading the values.
//!
//! Since the offsets are in order, no seeking is required after the values are
//! read, since the stream will already be pointed at the end of the values
//! section.
//!
//! ```
//! # use binrw::{args, BinRead, BinReaderExt, io::{Cursor, SeekFrom}};
//! use binrw::file_ptr::parse_from_iter;
//!
//! #[derive(BinRead)]
//! #[br(big)]
//! struct Object {
//!     count: u16,
//!     #[br(args { count: count.into() })]
//!     offsets: Vec<u16>,
//!     #[br(
//!         parse_with = parse_from_iter(offsets.iter().copied()),
//!         seek_before(SeekFrom::Start(0))
//!     )]
//!     values: Vec<u8>,
//!     extra: u16,
//! }
//!
//! # let mut x = Cursor::new(b"\0\x02\0\x06\0\x07\x04\x03\xff\xff");
//! # let x = Object::read(&mut x).unwrap();
//! # assert_eq!(x.values, &[4, 3]);
//! # assert_eq!(x.extra, 0xffff);
//! ```
//!
//! ## Using a function to lazily load values
//!
//! In this example, only the offset table is parsed. Values pointed to by the
//! offset table are loaded on demand by calling `Object::get` as needed at
//! runtime.
//!
//! ```
//! # use binrw::{args, BinRead, BinResult, BinReaderExt, helpers::until_eof, io::{Cursor, Read, Seek, SeekFrom}};
//!
//! #[derive(BinRead)]
//! # #[derive(Debug, Eq, PartialEq)]
//! #[br(big)]
//! struct Item(u8);
//!
//! #[derive(BinRead)]
//! #[br(big, stream = s)]
//! struct Object {
//!     count: u16,
//!     #[br(args { count: count.into() })]
//!     offsets: Vec<u16>,
//!     #[br(try_calc = s.stream_position())]
//!     data_offset: u64,
//! }
//!
//! impl Object {
//!     pub fn get<R: Read + Seek>(&self, source: &mut R, index: usize) -> Option<BinResult<Item>> {
//!         self.offsets.get(index).map(|offset| {
//!             let offset = self.data_offset + u64::from(*offset);
//!             source.seek(SeekFrom::Start(offset))?;
//!             Item::read(source)
//!         })
//!     }
//! }
//!
//! # let mut s = Cursor::new(b"\0\x02\0\x01\0\0\x03\x04");
//! # let x = Object::read(&mut s).unwrap();
//! # assert!(matches!(x.get(&mut s, 0), Some(Ok(Item(4)))));
//! # assert!(matches!(x.get(&mut s, 1), Some(Ok(Item(3)))));
//! # assert!(matches!(x.get(&mut s, 2), None));
//! ```

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
/// the value type `T` is the value at that offset. [Dereferencing] a `FilePtr`
/// yields the pointed-to value. When deriving `BinRead`, the
/// [offset](crate::docs::attribute#offset) directive can be used to adjust the
/// offset before the pointed-to value is read.
///
/// `FilePtr` is not efficient when reading offset tables; see the
/// [module documentation](binrw::file_ptr) for more information.
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use binrw::{helpers::read_u24, prelude::*};
    /// use binrw::FilePtr16;
    ///
    /// #[derive(BinRead)]
    /// struct Test {
    ///     #[br(parse_with = FilePtr16::parse_with(read_u24))]
    ///     value: u32
    /// }
    ///
    /// let mut data = binrw::io::Cursor::new(b"\x02\x00\x07\x0f\x10");
    /// let result = Test::read_le(&mut data).unwrap();
    /// assert_eq!(result.value, 0x100f07);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use binrw::{helpers::read_u24, prelude::*};
    /// use binrw::FilePtr16;
    ///
    /// #[derive(BinRead)]
    /// struct Test {
    ///     #[br(parse_with = FilePtr16::with(read_u24))]
    ///     value: FilePtr16<u32>
    /// }
    ///
    /// let mut data = binrw::io::Cursor::new(b"\x02\x00\x07\x0f\x10");
    /// let result = Test::read_le(&mut data).unwrap();
    /// assert_eq!(result.value.ptr, 2);
    /// assert_eq!(result.value.value, 0x100f07);
    /// ```
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

impl<Ptr, Value> Deref for FilePtr<Ptr, Value>
where
    Ptr: IntoSeekFrom,
{
    type Target = Value;

    /// Dereferences the value stored by `FilePtr`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use binrw::{prelude::*};
    /// use binrw::FilePtr16;
    ///
    /// #[derive(BinRead)]
    /// struct Test {
    ///     value: FilePtr16<u16>
    /// }
    ///
    /// let mut data = binrw::io::Cursor::new(b"\x02\x00\x01\x00");
    /// let result = Test::read_le(&mut data).unwrap();
    /// assert_eq!(result.value.ptr, 2);
    /// assert_eq!(result.value.value, 1);
    /// assert_eq!(*result.value, 1);
    /// ```
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<Ptr, Value> DerefMut for FilePtr<Ptr, Value>
where
    Ptr: IntoSeekFrom,
{
    /// Mutably dereferences the value stored by `FilePtr`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use binrw::{prelude::*};
    /// use binrw::FilePtr16;
    ///
    /// #[derive(BinRead)]
    /// struct Test {
    ///     value: FilePtr16<u16>
    /// }
    ///
    /// let mut data = binrw::io::Cursor::new(b"\x02\x00\x01\x00");
    /// let mut result = Test::read_le(&mut data).unwrap();
    /// assert_eq!(result.value.ptr, 2);
    /// assert_eq!(result.value.value, 1);
    /// *result.value = 42;
    /// assert_eq!(result.value.value, 42);
    /// ```
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

/// Creates a parser that reads a collection of values from an iterator of
/// file offsets using the [`BinRead`] implementation of `Value`.
///
/// Offsets are treated as relative to the position of the reader when
/// parsing begins. Use the [`seek_before`] directive to reposition the
/// stream in this case.
///
/// See the [module documentation](binrw::file_ptr) for more information on how
/// use `parse_from_iter`.
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
/// See the [module documentation](binrw::file_ptr) for more information on how
/// to use `parse_from_iter_with`.
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

/// A trait to convert from an integer into [`SeekFrom::Current`].
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
