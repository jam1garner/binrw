//! Helper functions for reading data.

use crate::{
    io::{ErrorKind::UnexpectedEof, Read, Seek},
    BinRead, BinReaderExt, BinResult, ReadOptions, VecArgs,
};
#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

/// A helper for more efficiently mass-reading bytes.
///
/// # Examples
///
/// ```
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
    T: BinRead<Args = Arg>,
    Reader: Read + Seek,
    CondFn: Fn(&T) -> bool,
    Arg: Clone,
    Ret: core::iter::FromIterator<T>,
{
    move |reader, ro, args| {
        let mut result = Vec::new();
        let mut last = reader.read_type_args(ro.endian, args.clone())?;

        while !cond(&last) {
            result.push(last);
            last = reader.read_type_args(ro.endian, args.clone())?;
        }

        result.push(last);

        Ok(result.into_iter().collect())
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
    T: BinRead<Args = Arg>,
    Reader: Read + Seek,
    CondFn: Fn(&T) -> bool,
    Arg: Clone,
    Ret: core::iter::FromIterator<T>,
{
    move |reader, ro, args| {
        let mut result = Vec::new();
        let mut last = reader.read_type_args(ro.endian, args.clone())?;

        while !cond(&last) {
            result.push(last);
            last = reader.read_type_args(ro.endian, args.clone())?;
        }

        Ok(result.into_iter().collect())
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
pub fn until_eof<R, T, Arg, Ret>(reader: &mut R, ro: &ReadOptions, args: Arg) -> BinResult<Ret>
where
    T: BinRead<Args = Arg>,
    R: Read + Seek,
    Arg: Clone,
    Ret: core::iter::FromIterator<T>,
{
    let mut result = Vec::new();
    let mut last = reader.read_type_args(ro.endian, args.clone());

    while !matches!(&last, Err(crate::Error::Io(err)) if err.kind() == UnexpectedEof) {
        last = match last {
            Ok(x) => {
                result.push(x);
                reader.read_type_args(ro.endian, args.clone())
            }
            Err(err) => return Err(err),
        }
    }

    Ok(result.into_iter().collect())
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
    T: BinRead<Args = Arg>,
    R: Read + Seek,
    Arg: Clone,
    Ret: core::iter::FromIterator<T>,
{
    move |reader, ro, args| {
        (0..n)
            .map(|_| reader.read_type_args(ro.endian, args.clone()))
            .collect()
    }
}
