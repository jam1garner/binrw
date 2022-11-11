//! Helper functions for reading data.

use crate::{
    io::{self, Read, Seek},
    BinRead, BinResult, Error, ReadOptions,
};
use alloc::vec::Vec;
use core::iter::from_fn;

/// Creates a parser that reads items into a collection until a condition is
/// met. The terminal item is added to the collection.
///
/// This helper can be used to read into any collection type that implements
/// [`FromIterator`].
///
/// # Examples
///
/// ```
/// # use binrw::{BinRead, helpers::until, io::Cursor, BinReaderExt};
/// #[derive(BinRead)]
/// struct NullTerminated {
///     #[br(parse_with = until(|&byte| byte == 0))]
///     data: Vec<u8>,
/// }
///
/// # let mut x = Cursor::new(b"\x01\x02\x03\x04\0");
/// # let x: NullTerminated = x.read_be().unwrap();
/// # assert_eq!(x.data, &[1, 2, 3, 4, 0]);
/// ```
pub fn until<Reader, T, CondFn, Arg, Ret>(
    cond: CondFn,
) -> impl Fn(&mut Reader, &ReadOptions, Arg) -> BinResult<Ret>
where
    T: BinRead<Args = Arg>,
    Reader: Read + Seek,
    CondFn: Fn(&T) -> bool,
    Arg: Clone,
    Ret: FromIterator<T>,
{
    until_with(cond, default_reader)
}

/// Creates a parser that uses a given function to read items into a collection
/// until a condition is met. The terminal item is added to the collection.
///
/// The given `read` function should return one item each time it is called.
///
/// This helper can be used to read into any collection type that implements
/// [`FromIterator`].
///
/// # Examples
///
/// Reading a two-dimensional `VecDeque` by combining [`until_with`] and
/// [`count`]:
///
/// ```
/// # use binrw::{BinRead, helpers::{until, until_with, count}, io::Cursor, BinReaderExt};
/// # use std::collections::VecDeque;
/// #[derive(BinRead)]
/// struct NullTerminated {
///     #[br(parse_with = until_with(|bytes| bytes == &[0, 0], count(2)))]
///     data: VecDeque<VecDeque<u8>>,
/// }
///
/// # let mut x = Cursor::new(b"\x01\x02\x03\x04\0\0");
/// # let x: NullTerminated = x.read_be().unwrap();
/// # assert_eq!(x.data, &[[1, 2], [3, 4], [0, 0]]);
/// ```
pub fn until_with<Reader, T, CondFn, Arg, ReadFn, Ret>(
    cond: CondFn,
    read: ReadFn,
) -> impl Fn(&mut Reader, &ReadOptions, Arg) -> BinResult<Ret>
where
    Reader: Read + Seek,
    CondFn: Fn(&T) -> bool,
    Arg: Clone,
    ReadFn: Fn(&mut Reader, &ReadOptions, Arg) -> BinResult<T>,
    Ret: FromIterator<T>,
{
    move |reader, ro, args| {
        let mut last = false;
        from_fn(|| {
            if last {
                None
            } else {
                match read(reader, ro, args.clone()) {
                    Ok(value) => {
                        if cond(&value) {
                            last = true;
                        }
                        Some(Ok(value))
                    }
                    err => Some(err),
                }
            }
        })
        .fuse()
        .collect()
    }
}

/// Creates a parser that reads items into a collection until a condition is
/// met. The terminal item is discarded.
///
/// This helper can be used to read into any collection type that implements
/// [`FromIterator`].
///
/// # Examples
///
/// ```
/// # use binrw::{BinRead, helpers::until_exclusive, io::Cursor, BinReaderExt};
/// #[derive(BinRead)]
/// struct NullTerminated {
///     #[br(parse_with = until_exclusive(|&byte| byte == 0))]
///     data: Vec<u8>,
/// }
///
/// # let mut x = Cursor::new(b"\x01\x02\x03\x04\0");
/// # let x: NullTerminated = x.read_be().unwrap();
/// # assert_eq!(x.data, &[1, 2, 3, 4]);
/// ```
pub fn until_exclusive<Reader, T, CondFn, Arg, Ret>(
    cond: CondFn,
) -> impl Fn(&mut Reader, &ReadOptions, Arg) -> BinResult<Ret>
where
    T: BinRead<Args = Arg>,
    Reader: Read + Seek,
    CondFn: Fn(&T) -> bool,
    Arg: Clone,
    Ret: FromIterator<T>,
{
    until_exclusive_with(cond, default_reader)
}

/// Creates a parser that uses a given function to read items into a collection
/// until a condition is met. The terminal item is discarded.
///
/// The given `read` function should return one item each time it is called.
///
/// This helper can be used to read into any collection type that implements
/// [`FromIterator`].
///
/// # Examples
///
/// Reading a two-dimensional `VecDeque` by combining [`until_exclusive_with`]
/// and [`count`]:
///
/// ```
/// # use binrw::{BinRead, helpers::{until_exclusive, until_exclusive_with, count}, io::Cursor, BinReaderExt};
/// # use std::collections::VecDeque;
/// #[derive(BinRead)]
/// struct NullTerminated {
///     #[br(parse_with = until_exclusive_with(|bytes| bytes == &[0, 0], count(2)))]
///     data: VecDeque<VecDeque<u8>>,
/// }
///
/// # let mut x = Cursor::new(b"\x01\x02\x03\x04\0\0");
/// # let x: NullTerminated = x.read_be().unwrap();
/// # assert_eq!(x.data, &[[1, 2], [3, 4]]);
/// ```
pub fn until_exclusive_with<Reader, T, CondFn, Arg, ReadFn, Ret>(
    cond: CondFn,
    read: ReadFn,
) -> impl Fn(&mut Reader, &ReadOptions, Arg) -> BinResult<Ret>
where
    Reader: Read + Seek,
    CondFn: Fn(&T) -> bool,
    Arg: Clone,
    ReadFn: Fn(&mut Reader, &ReadOptions, Arg) -> BinResult<T>,
    Ret: FromIterator<T>,
{
    move |reader, ro, args| {
        from_fn(|| match read(reader, ro, args.clone()) {
            Ok(value) => {
                if cond(&value) {
                    None
                } else {
                    Some(Ok(value))
                }
            }
            err => Some(err),
        })
        .fuse()
        .collect()
    }
}

/// Creates a parser that reads items into a collection until the end of the
/// input stream.
///
/// This helper can be used to read into any collection type that implements
/// [`FromIterator`].
///
/// # Errors
///
/// If reading fails for a reason other than reaching the end of the input, an
/// [`Error`](crate::Error) variant will be returned.
///
/// # Examples
///
/// ```
/// # use binrw::{BinRead, helpers::until_eof, io::Cursor, BinReaderExt};
/// #[derive(BinRead)]
/// struct EntireFile {
///     #[br(parse_with = until_eof)]
///     data: Vec<u8>,
/// }
///
/// # let mut x = Cursor::new(b"\x01\x02\x03\x04");
/// # let x: EntireFile = x.read_be().unwrap();
/// # assert_eq!(x.data, &[1, 2, 3, 4]);
/// ```
pub fn until_eof<Reader, T, Arg, Ret>(
    reader: &mut Reader,
    ro: &ReadOptions,
    args: Arg,
) -> BinResult<Ret>
where
    T: BinRead<Args = Arg>,
    Reader: Read + Seek,
    Arg: Clone,
    Ret: FromIterator<T>,
{
    until_eof_with(default_reader)(reader, ro, args)
}

/// Creates a parser that uses a given function to read items into a collection
/// until the end of the input stream.
///
/// The given `read` function should return one item each time it is called.
///
/// This helper can be used to read into any collection type that implements
/// [`FromIterator`].
///
/// # Errors
///
/// If reading fails for a reason other than reaching the end of the input, an
/// [`Error`](crate::Error) variant will be returned.
///
/// # Examples
///
/// Reading a two-dimensional `VecDeque` by combining [`until_eof_with`] and
/// [`count`]:
///
/// ```
/// # use binrw::{BinRead, helpers::{until_eof, until_eof_with, count}, io::Cursor, BinReaderExt};
/// # use std::collections::VecDeque;
/// #[derive(BinRead)]
/// struct EntireFile {
///     #[br(parse_with = until_eof_with(count(2)))]
///     data: VecDeque<VecDeque<u8>>,
/// }
///
/// # let mut x = Cursor::new(b"\x01\x02\x03\x04");
/// # let x: EntireFile = x.read_be().unwrap();
/// # assert_eq!(x.data, &[[1, 2], [3, 4]]);
/// ```
pub fn until_eof_with<Reader, T, Arg, ReadFn, Ret>(
    read: ReadFn,
) -> impl Fn(&mut Reader, &ReadOptions, Arg) -> BinResult<Ret>
where
    Reader: Read + Seek,
    Arg: Clone,
    ReadFn: Fn(&mut Reader, &ReadOptions, Arg) -> BinResult<T>,
    Ret: FromIterator<T>,
{
    move |reader, ro, args| {
        from_fn(|| match read(reader, ro, args.clone()) {
            ok @ Ok(_) => Some(ok),
            Err(err) if err.is_eof() => None,
            err => Some(err),
        })
        .fuse()
        .collect()
    }
}

/// Creates a parser that reads N items into a collection.
///
/// This helper is similar to using `#[br(count = N)]` with [`Vec`], but is more
/// generic so can be used to read into any collection type that implements
/// [`FromIterator`].
///
/// # Examples
///
/// ```
/// # use binrw::{BinRead, helpers::count, io::Cursor, BinReaderExt};
/// # use std::collections::VecDeque;
/// #[derive(BinRead)]
/// struct CountBytes {
///     len: u8,
///
///     #[br(parse_with = count(len as usize))]
///     data: VecDeque<u8>,
/// }
///
/// # let mut x = Cursor::new(b"\x03\x01\x02\x03");
/// # let x: CountBytes = x.read_be().unwrap();
/// # assert_eq!(x.data, &[1, 2, 3]);
/// ```
pub fn count<R, T, Arg, Ret>(n: usize) -> impl Fn(&mut R, &ReadOptions, Arg) -> BinResult<Ret>
where
    T: BinRead<Args = Arg>,
    R: Read + Seek,
    Arg: Clone,
    Ret: FromIterator<T> + 'static,
{
    count_with(n, default_reader)
}

/// Creates a parser that uses a given function to read N items into a
/// collection.
///
/// The given `read` function should return one item each time it is called.
///
/// This helper is similar to using `#[br(count = N)]` with [`Vec`], but is more
/// generic so can be used to read into any collection type that implements
/// [`FromIterator`].
///
/// # Examples
///
/// Reading a two-dimensional `VecDeque` by combining [`count_with`] and
/// [`count`]:
///
/// ```
/// # use binrw::{BinRead, helpers::count, helpers::count_with, io::Cursor, BinReaderExt};
/// # use std::collections::VecDeque;
/// #[derive(BinRead)]
/// struct CountBytes {
///     len: u8,
///
///     #[br(parse_with = count_with(len as usize, count(2)))]
///     data: VecDeque<VecDeque<u8>>,
/// }
///
/// # let mut x = Cursor::new(b"\x02\x01\x02\x03\x04");
/// # let x: CountBytes = x.read_be().unwrap();
/// # assert_eq!(x.data, &[[1, 2], [3, 4]]);
pub fn count_with<R, T, Arg, ReadFn, Ret>(
    n: usize,
    read: ReadFn,
) -> impl Fn(&mut R, &ReadOptions, Arg) -> BinResult<Ret>
where
    R: Read + Seek,
    Arg: Clone,
    ReadFn: Fn(&mut R, &ReadOptions, Arg) -> BinResult<T>,
    Ret: FromIterator<T> + 'static,
{
    move |reader, ro, args| {
        let mut container = core::iter::empty::<T>().collect::<Ret>();

        vec_fast_int!(try (i8 i16 u16 i32 u32 i64 u64 i128 u128) using (container, reader, ro.endian(), n) else {
            // This extra branch for `Vec<u8>` makes it faster than
            // `vec_fast_int`, but *only* because `vec_fast_int` is not allowed
            // to use unsafe code to eliminate the unnecessary zero-fill.
            // Otherwise, performance would be identical and it could be
            // deleted.
            if let Some(bytes) = <dyn core::any::Any>::downcast_mut::<Vec<u8>>(&mut container) {
                bytes.reserve_exact(n);
                let byte_count = reader
                    .take(n.try_into().map_err(not_enough_bytes)?)
                    .read_to_end(bytes)?;

                if byte_count == n {
                    Ok(container)
                } else {
                    Err(not_enough_bytes(()))
                }
            } else {
                core::iter::repeat_with(|| read(reader, ro, args.clone()))
                .take(n)
                .collect()
            }
        })
    }
}

/// Creates a parser that uses an iterator to read into a collection, using
/// each item of the iterator as an argument.
///
/// # Examples
///
/// Reading an object containing header data followed by body data:
///
/// ```
/// # use binrw::{args, BinRead, helpers::from_iterator_args, io::Cursor, BinReaderExt};
/// #[derive(BinRead)]
/// #[br(big)]
/// struct Header {
///     count: u16,
///
///     #[br(args { count: count.into() })]
///     sizes: Vec<u16>,
/// }
///
/// #[derive(BinRead)]
/// #[br(big)]
/// struct Object {
///     header: Header,
///     #[br(parse_with = from_iterator_args(header.sizes.iter().map(|size| -> <Vec<u8> as BinRead>::Args {
///         args! { count: (*size).into() }
///     })))]
///     segments: Vec<Vec<u8>>,
/// }
///
/// # let mut x = Cursor::new(b"\0\x02\0\x01\0\x02\x03\x04\x05");
/// # let x = Object::read(&mut x).unwrap();
/// # assert_eq!(x.segments, &[vec![3], vec![4, 5]]);
pub fn from_iterator_args<R, T, Arg, Ret, It>(
    it: It,
) -> impl FnOnce(&mut R, &ReadOptions, ()) -> BinResult<Ret>
where
    T: BinRead<Args = Arg>,
    R: Read + Seek,
    Arg: Clone,
    Ret: FromIterator<T> + 'static,
    It: Iterator<Item = Arg>,
{
    from_iterator_args_with(it, default_reader)
}

/// Creates a parser that uses an iterator to read into a collection, using
/// each item of the iterator as an argument passed to the given read function.
///
/// # Examples
///
/// Reading an object containing header data followed by body data:
///
/// ```
/// # use binrw::{args, BinRead, helpers::from_iterator_args_with, io::Cursor, BinReaderExt};
/// #[derive(BinRead)]
/// #[br(big)]
/// struct Header {
///     count: u16,
///
///     #[br(args { count: count.into() })]
///     sizes: Vec<u16>,
/// }
///
/// #[derive(BinRead)]
/// #[br(big)]
/// struct Object {
///     header: Header,
///     #[br(parse_with = from_iterator_args_with(header.sizes.iter(), |reader, options, size| {
///         Vec::<u8>::read_options(reader, options, args! { count: (*size).into() })
///     }))]
///     segments: Vec<Vec<u8>>,
/// }
///
/// # let mut x = Cursor::new(b"\0\x02\0\x01\0\x02\x03\x04\x05");
/// # let x = Object::read(&mut x).unwrap();
/// # assert_eq!(x.segments, &[vec![3], vec![4, 5]]);
pub fn from_iterator_args_with<Reader, T, Arg, Ret, It, ReadFn>(
    it: It,
    read: ReadFn,
) -> impl FnOnce(&mut Reader, &ReadOptions, ()) -> BinResult<Ret>
where
    T: BinRead,
    Reader: Read + Seek,
    Arg: Clone,
    Ret: FromIterator<T> + 'static,
    It: Iterator<Item = Arg>,
    ReadFn: Fn(&mut Reader, &ReadOptions, Arg) -> BinResult<T>,
{
    move |reader, options, _| it.map(|arg| read(reader, options, arg)).collect()
}

fn default_reader<R: Read + Seek, Arg: Clone, T: BinRead<Args = Arg>>(
    reader: &mut R,
    options: &ReadOptions,
    args: T::Args,
) -> BinResult<T> {
    let mut value = T::read_options(reader, options, args.clone())?;
    value.after_parse(reader, options, args)?;
    Ok(value)
}

fn not_enough_bytes<T>(_: T) -> Error {
    Error::Io(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "not enough bytes in reader",
    ))
}

macro_rules! vec_fast_int {
    (try ($($Ty:ty)+) using ($list:expr, $reader:expr, $endian:expr, $count:expr) else { $($else:tt)* }) => {
        $(if let Some(list) = <dyn core::any::Any>::downcast_mut::<Vec<$Ty>>(&mut $list) {
            let mut start = 0;
            let mut remaining = $count;
            // Allocating and reading from the source in chunks is done to keep
            // a bad `count` from causing huge memory allocations that are
            // doomed to fail
            while remaining != 0 {
                // Using a similar strategy as std `default_read_to_end` to
                // leverage the memory growth strategy of the underlying Vec
                // implementation (in std this will be exponential) using a
                // minimum byte allocation
                const GROWTH: usize = 32 / core::mem::size_of::<$Ty>();
                list.reserve(remaining.min(GROWTH.max(1)));

                let items_to_read = remaining.min(list.capacity() - start);
                let end = start + items_to_read;

                // In benchmarks, this resize decreases performance by 27–40%
                // relative to using `unsafe` to write directly to uninitialised
                // memory, but nobody ever got fired for buying IBM
                list.resize(end, 0);
                $reader.read_exact(&mut bytemuck::cast_slice_mut::<_, u8>(&mut list[start..end]))?;

                remaining -= items_to_read;
                start += items_to_read;
            }

            if
                core::mem::size_of::<$Ty>() != 1
                && (
                    (cfg!(target_endian = "big") && $endian == crate::Endian::Little)
                    || (cfg!(target_endian = "little") && $endian == crate::Endian::Big)
                )
            {
                for value in list.iter_mut() {
                    *value = value.swap_bytes();
                }
            }
            Ok($list)
        } else)* {
            $($else)*
        }
    }
}

use vec_fast_int;
