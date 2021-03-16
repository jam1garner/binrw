use crate::{
    alloc::string::{FromUtf8Error, FromUtf16Error},
    io::{Read, Seek},
    BinRead,
    ReadOptions,
    BinResult,
};

// #[cfg(feature = "std")]
// use std::{
//     ffi::CString,
// };

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec};
use core::num::{NonZeroU8, NonZeroU16};

impl BinRead for Vec<NonZeroU8> {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, _: &ReadOptions, _: Self::Args) -> BinResult<Self>
    {
        reader
            .bytes()
            .take_while(|x| !matches!(x, Ok(0)))
            .map(|x| Ok(x.map(|byte| unsafe { NonZeroU8::new_unchecked(byte) })?))
            .collect()
    }
}

/// A null terminated UTF-8 string designed to make reading any null-terminated data easier.
///
/// **Note:** Result does not include the null, but the null is consumed from the Reader.
///
/// ```rust
/// use binrw::{BinRead, BinReaderExt, NullString, io::Cursor};
///
/// let mut null_separated_strings = Cursor::new(b"null terminated strings? in my system's language?\0no thanks\0");
///
/// assert_eq!(
///     null_separated_strings.read_be::<NullString>().unwrap().into_string(),
///     "null terminated strings? in my system's language?"
/// );
///
/// assert_eq!(
///     null_separated_strings.read_be::<NullString>().unwrap().into_string(),
///     "no thanks"
/// );
/// ```
#[derive(Clone, PartialEq, Default)]
pub struct NullString(pub Vec<u8>);

/// A null terminated UTF-16 string designed to make reading any 16 bit wide null-terminated data easier.
///
/// **Note:** Does not include the null.
///
/// **Note:** This is endian dependent on a per-character basis. Will read `u16`s until a `0u16` is found.
///
/// ```rust
/// use binrw::{BinRead, BinReaderExt, NullWideString, io::Cursor};
///
/// const WIDE_STRINGS: &[u8] = b"w\0i\0d\0e\0 \0s\0t\0r\0i\0n\0g\0s\0\0\0";
/// const ARE_ENDIAN_DEPENDENT: &[u8] = b"\0a\0r\0e\0 \0e\0n\0d\0i\0a\0n\0 \0d\0e\0p\0e\0n\0d\0e\0n\0t\0\0";
///
/// let mut wide_strings = Cursor::new(WIDE_STRINGS);
/// let mut are_endian_dependent = Cursor::new(ARE_ENDIAN_DEPENDENT);
///
/// assert_eq!(
///     // notice: read_le
///     wide_strings.read_le::<NullWideString>().unwrap().into_string(),
///     "wide strings"
/// );
///
/// assert_eq!(
///     // notice: read_be
///     are_endian_dependent.read_be::<NullWideString>().unwrap().into_string(),
///     "are endian dependent"
/// );
/// ```
#[derive(Clone, PartialEq, Default)]
pub struct NullWideString(pub Vec<u16>);

impl NullString {
    pub fn into_string(self) -> String {
        String::from_utf8_lossy(&self.0).into()
    }

    pub fn into_string_lossless(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.0)
    }
}

impl NullWideString {
    pub fn into_string(self) -> String {
        String::from_utf16_lossy(&self.0)
    }

    pub fn into_string_lossless(self) -> Result<String, FromUtf16Error> {
        String::from_utf16(&self.0)
    }
}

impl From<Vec<NonZeroU16>> for NullWideString {
    fn from(v: Vec<NonZeroU16>) -> NullWideString {
        NullWideString(v.into_iter().map(|x| x.get()).collect())
    }
}

impl From<Vec<NonZeroU8>> for NullString {
    fn from(v: Vec<NonZeroU8>) -> Self {
        NullString(v.into_iter().map(|x| x.get()).collect())
    }
}

impl From<NullWideString> for Vec<u16> {
    fn from(s: NullWideString) -> Self {
        s.0
    }
}

impl From<NullString> for Vec<u8> {
    fn from(s: NullString) -> Self {
        s.0
    }
}

impl BinRead for Vec<NonZeroU16> {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: Self::Args)
        -> BinResult<Self>
    {
        let mut values = vec![];

        loop {
            let val = <u16>::read_options(reader, options, ())?;
            if val == 0 {
                return Ok(values)
            }
            values.push(unsafe { NonZeroU16::new_unchecked(val) });
        }
    }
}

impl BinRead for NullWideString {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args)
        -> BinResult<Self>
    {
        // https://github.com/rust-lang/rust-clippy/issues/6447
        #[allow(clippy::unit_arg)]
        <Vec<NonZeroU16>>::read_options(reader, &options, args)
            .map(|chars| chars.into())
    }
}

impl BinRead for NullString {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args)
        -> BinResult<Self>
    {
        // https://github.com/rust-lang/rust-clippy/issues/6447
        #[allow(clippy::unit_arg)]
        <Vec<NonZeroU8>>::read_options(reader, options, args)
            .map(|chars| chars.into())
    }
}

use core::fmt;

impl fmt::Debug for NullString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NullString({:?})", self.clone().into_string())
    }
}

impl fmt::Debug for NullWideString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NullWideString({:?})", self.clone().into_string())
    }
}

impl core::ops::Deref for NullString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::Deref for NullWideString {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ToString for NullString {
    fn to_string(&self) -> String {
        core::str::from_utf8(&self).unwrap().to_string()
    }
}

impl ToString for NullWideString {
    fn to_string(&self) -> String {
        String::from_utf16_lossy(self)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn null_wide_strings() {
        use crate::{BinReaderExt, NullWideString, io::Cursor};

        const WIDE_STRINGS: &[u8] = b"w\0i\0d\0e\0 \0s\0t\0r\0i\0n\0g\0s\0\0\0";
        const ARE_ENDIAN_DEPENDENT: &[u8] = b"\0a\0r\0e\0 \0e\0n\0d\0i\0a\0n\0 \0d\0e\0p\0e\0n\0d\0e\0n\0t\0\0";

        let mut wide_strings = Cursor::new(WIDE_STRINGS);
        let mut are_endian_dependent = Cursor::new(ARE_ENDIAN_DEPENDENT);

        let wide_strings: NullWideString = wide_strings.read_le().unwrap();
        let are_endian_dependent: NullWideString = are_endian_dependent.read_be().unwrap();

        assert_eq!(
            // notice: read_le
            wide_strings.into_string(),
            "wide strings"
        );

        assert_eq!(
            // notice: read_be
            are_endian_dependent.into_string(),
            "are endian dependent"
        );
    }

    #[test]
    fn null_strings() {
        use crate::{BinReaderExt, NullString, io::Cursor};

        let mut null_separated_strings = Cursor::new(b"null terminated strings? in my system's language?\0no thanks\0");

        assert_eq!(
            null_separated_strings.read_be::<NullString>().unwrap().into_string(),
            "null terminated strings? in my system's language?"
        );

        assert_eq!(
            null_separated_strings.read_be::<NullString>().unwrap().into_string(),
            "no thanks"
        );
    }
}
