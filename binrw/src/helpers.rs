//! Helper functions for reading data.

use crate::{
    arg_type,
    io::{self, Read, Seek},
    BinRead, BinResult, Error, ReadOptions, VecArgs,
};
use core::convert::TryInto;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

/// A helper for more efficiently mass-reading bytes.
///
/// # Examples
///
/// ```
/// #![allow(deprecated)]
/// # use binrw::{BinRead, helpers::read_bytes, io::Cursor, BinReaderExt};
/// #[derive(BinRead)]
/// struct BunchaBytes {
///     #[br(count = 5)]
///     data: Vec<u8>
/// }
///
/// # let mut x = Cursor::new(b"\0\x01\x02\x03\x04");
/// # let x: BunchaBytes = x.read_be().unwrap();
/// # assert_eq!(x.data, &[0, 1, 2, 3, 4]);
/// ```
#[deprecated(since = "0.2.0", note = "Use Vec<u8> instead.")]
pub fn read_bytes<R: Read + Seek>(
    reader: &mut R,
    _options: &ReadOptions,
    args: VecArgs<()>,
) -> BinResult<Vec<u8>> {
    let mut buf = vec![0; args.count];
    reader.read_exact(&mut buf)?;

    Ok(buf)
}

/// Read items until a condition is met. The final item will be included.
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
    T: BinRead<Args = arg_type!(Arg)>,
    Reader: Read + Seek,
    CondFn: Fn(&T) -> bool,
    Arg: Clone,
    Ret: core::iter::FromIterator<T>,
{
    let read = |reader: &mut Reader, ro: &ReadOptions, args: Arg| {
        let mut value = T::read_options(reader, ro, args.clone())?;
        value.after_parse(reader, ro, args)?;
        Ok(value)
    };

    until_with(cond, read)
}

/// Do the same as [until](binrw::helpers::until) with a custom parsing function for the inner type.
///
/// # Examples
///
/// This example shows how to read lists of two elements until a condition is met using [until_with](binrw::helpers::until_with) coupled with [count](binrw::helpers::count).
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
    Ret: core::iter::FromIterator<T>,
{
    move |reader, ro, args| {
        let mut last_cond = true;
        let mut last_error = false;
        core::iter::repeat_with(|| read(reader, ro, args.clone()))
            .take_while(|result| {
                let cont = last_cond && !last_error; //keep the first error we get
                if let Ok(val) = result {
                    last_cond = !cond(val);
                } else {
                    last_error = true;
                }
                cont
            })
            .collect()
    }
}

/// Read items until a condition is met. The last item will *not* be included.
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
    T: BinRead<Args = arg_type!(Arg)>,
    Reader: Read + Seek,
    CondFn: Fn(&T) -> bool,
    Arg: Clone,
    Ret: core::iter::FromIterator<T>,
{
    let read = |reader: &mut Reader, ro: &ReadOptions, args: Arg| {
        let mut value = T::read_options(reader, ro, args.clone())?;
        value.after_parse(reader, ro, args)?;
        Ok(value)
    };
    until_exclusive_with(cond, read)
}

/// Do the same as [until_exclusive](binrw::helpers::until_exclusive) with a custom parsing function for the inner type.
///
/// # Examples
///
/// This example shows how to read lists of two elements until a condition is met using [until_exclusive_with](binrw::helpers::until_exclusive_with) coupled with [count](binrw::helpers::count).
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
    Ret: core::iter::FromIterator<T>,
{
    move |reader, ro, args| {
        let mut last_error = false;
        core::iter::repeat_with(|| read(reader, ro, args.clone()))
            .take_while(|result| {
                !last_error
                    && if let Ok(val) = result {
                        !cond(val)
                    } else {
                        last_error = true;
                        true //keep the first error we get
                    }
            })
            .collect()
    }
}

/// Read items until the end of the file is hit.
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
    T: BinRead<Args = arg_type!(Arg)>,
    Reader: Read + Seek,
    Arg: Clone,
    Ret: core::iter::FromIterator<T>,
{
    let read = |reader: &mut Reader, ro: &ReadOptions, args: Arg| {
        let mut value = T::read_options(reader, ro, args.clone())?;
        value.after_parse(reader, ro, args)?;
        Ok(value)
    };
    until_eof_with(read)(reader, ro, args)
}

/// Do the same as [until_eof](binrw::helpers::until_eof) with a custom parsing function for the inner type.
///
/// # Examples
///
/// This example shows how to read lists of two elements until the end of file using [until_eof_with](binrw::helpers::until_eof_with) coupled with [count](binrw::helpers::count).
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
    Ret: core::iter::FromIterator<T>,
{
    move |reader, ro, args| {
        let mut last_error = false;
        core::iter::repeat_with(|| read(reader, ro, args.clone()))
            .take_while(|result| {
                !last_error
                    && match result {
                        Ok(_) => true,
                        Err(e) if e.is_eof() => false,
                        Err(_) => {
                            last_error = true;
                            true //keep the first error we get
                        }
                    }
            })
            .collect()
    }
}

fn not_enough_bytes<T>(_: T) -> Error {
    Error::Io(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "not enough bytes in reader",
    ))
}

/// A helper similar to `#[br(count = N)]` which can be used with any collection.
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
    T: BinRead<Args = arg_type!(Arg)>,
    R: Read + Seek,
    Arg: Clone,
    Ret: core::iter::FromIterator<T> + 'static,
{
    move |reader, ro, args| {
        let mut container: Ret = core::iter::empty::<T>().collect();
        if let Some(bytes) = <dyn core::any::Any>::downcast_mut::<Vec<u8>>(&mut container) {
            bytes.reserve(n);
            let byte_count = reader
                .take(n.try_into().map_err(not_enough_bytes)?)
                .read_to_end(bytes)?;
            (byte_count == n)
                .then(|| container)
                .ok_or_else(|| not_enough_bytes(()))
        } else {
            let read = |reader: &mut R, ro: &ReadOptions, args: Arg| {
                let mut value = T::read_options(reader, ro, args.clone())?;
                value.after_parse(reader, ro, args)?;
                Ok(value)
            };
            count_with(n, read)(reader, ro, args)
        }
    }
}

/// Do the same as [count](binrw::helpers::count) with a custom parsing function for the inner type.
///
/// # Examples
///
/// This example shows how to read `len` lists of two elements using [count_with](binrw::helpers::count_with) coupled with [count](binrw::helpers::count).
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
    Ret: core::iter::FromIterator<T> + 'static,
{
    move |reader, ro, args| {
        core::iter::repeat_with(|| read(reader, ro, args.clone()))
            .take(n)
            .collect()
    }
}
