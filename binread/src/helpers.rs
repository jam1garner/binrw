use crate::{BinResult, ReadOptions, io::{Read, Seek}};

/// A helper for more efficiently mass-reading bytes
///
///## Example:
///
/// ```rust
/// # use binread::{BinRead, helpers::read_bytes, io::Cursor, BinReaderExt};
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
pub fn read_bytes<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: ()) -> BinResult<Vec<u8>> {
    let count = match options.count {
        Some(x) => x,
        None => panic!("Missing count for read_bytes")
    };
    let mut buf = vec![0; count];
    reader.read_exact(&mut buf)?;

    Ok(buf)
}
