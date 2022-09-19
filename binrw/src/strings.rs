//! Type definitions for string readers.

use crate::{
    alloc::string::{FromUtf16Error, FromUtf8Error},
    io::{Read, Seek, Write},
    BinRead, BinResult, BinWrite, ReadOptions,
};
use alloc::{string::String, vec, vec::Vec};
use core::fmt::{self, Write as _};

/// A null-terminated 8-bit string.
///
/// The null terminator is consumed and not included in the value.
///
/// ```
/// use binrw::{BinRead, BinReaderExt, NullString, io::Cursor};
///
/// let mut null_separated_strings = Cursor::new(b"null terminated strings? in my system's language?\0no thanks\0");
///
/// assert_eq!(
///     null_separated_strings.read_be::<NullString>().unwrap().to_string(),
///     "null terminated strings? in my system's language?"
/// );
///
/// assert_eq!(
///     null_separated_strings.read_be::<NullString>().unwrap().to_string(),
///     "no thanks"
/// );
/// ```
#[derive(Clone, Eq, PartialEq, Default)]
pub struct NullString(
    /// The raw byte string.
    pub Vec<u8>,
);

impl BinRead for NullString {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        let mut values = vec![];

        loop {
            let val = <u8>::read_options(reader, options, ())?;
            if val == 0 {
                return Ok(Self(values));
            }
            values.push(val);
        }
    }
}

impl BinWrite for NullString {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &crate::WriteOptions,
        args: Self::Args,
    ) -> BinResult<()> {
        self.0.write_options(writer, options, args)?;
        0u8.write_options(writer, options, args)?;

        Ok(())
    }
}

impl From<&str> for NullString {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<String> for NullString {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<NullString> for Vec<u8> {
    fn from(s: NullString) -> Self {
        s.0
    }
}

impl TryFrom<NullString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: NullString) -> Result<Self, Self::Error> {
        String::from_utf8(value.0)
    }
}

impl core::ops::Deref for NullString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for NullString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for NullString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NullString(\"")?;
        display_utf8(&self.0, f, str::escape_debug)?;
        write!(f, "\")")
    }
}

impl fmt::Display for NullString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf8(&self.0, f, str::chars)
    }
}

/// A null-terminated 16-bit string.
///
/// The null terminator must also be 16-bits, and is consumed and not included
/// in the value.
///
/// ```
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
///     wide_strings.read_le::<NullWideString>().unwrap().to_string(),
///     "wide strings"
/// );
///
/// assert_eq!(
///     // notice: read_be
///     are_endian_dependent.read_be::<NullWideString>().unwrap().to_string(),
///     "are endian dependent"
/// );
/// ```
#[derive(Clone, Eq, PartialEq, Default)]
pub struct NullWideString(
    /// The raw wide byte string.
    pub Vec<u16>,
);

impl BinRead for NullWideString {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        let mut values = vec![];

        loop {
            let val = <u16>::read_options(reader, options, ())?;
            if val == 0 {
                return Ok(Self(values));
            }
            values.push(val);
        }
    }
}

impl BinWrite for NullWideString {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &crate::WriteOptions,
        args: Self::Args,
    ) -> BinResult<()> {
        self.0.write_options(writer, options, args)?;
        0u16.write_options(writer, options, args)?;

        Ok(())
    }
}

impl From<NullWideString> for Vec<u16> {
    fn from(s: NullWideString) -> Self {
        s.0
    }
}

impl From<&str> for NullWideString {
    fn from(s: &str) -> Self {
        Self(s.encode_utf16().collect())
    }
}

impl From<String> for NullWideString {
    fn from(s: String) -> Self {
        Self(s.encode_utf16().collect())
    }
}

impl TryFrom<NullWideString> for String {
    type Error = FromUtf16Error;

    fn try_from(value: NullWideString) -> Result<Self, Self::Error> {
        String::from_utf16(&value.0)
    }
}

impl core::ops::Deref for NullWideString {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for NullWideString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for NullWideString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf16(&self.0, f, core::iter::once)
    }
}

impl fmt::Debug for NullWideString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NullWideString(\"")?;
        display_utf16(&self.0, f, char::escape_debug)?;
        write!(f, "\")")
    }
}

fn display_utf16<Transformer: Fn(char) -> O, O: Iterator<Item = char>>(
    input: &[u16],
    f: &mut fmt::Formatter<'_>,
    t: Transformer,
) -> fmt::Result {
    char::decode_utf16(input.iter().copied())
        .flat_map(|r| t(r.unwrap_or(char::REPLACEMENT_CHARACTER)))
        .try_for_each(|c| f.write_char(c))
}

fn display_utf8<'a, Transformer: Fn(&'a str) -> O, O: Iterator<Item = char> + 'a>(
    mut input: &'a [u8],
    f: &mut fmt::Formatter<'_>,
    t: Transformer,
) -> fmt::Result {
    // Adapted from <https://doc.rust-lang.org/std/str/struct.Utf8Error.html>
    loop {
        match core::str::from_utf8(input) {
            Ok(valid) => {
                t(valid).try_for_each(|c| f.write_char(c))?;
                break;
            }
            Err(error) => {
                let (valid, after_valid) = input.split_at(error.valid_up_to());

                t(core::str::from_utf8(valid).unwrap()).try_for_each(|c| f.write_char(c))?;
                f.write_char(char::REPLACEMENT_CHARACTER)?;

                if let Some(invalid_sequence_length) = error.error_len() {
                    input = &after_valid[invalid_sequence_length..];
                } else {
                    break;
                }
            }
        }
    }
    Ok(())
}


/// An 8-bit string with pre-computed length.
///
/// The null terminator is consumed and included if the string is null-terminated.
///
/// ```
/// use binrw::{prelude::*, FixedLenString, io::Cursor};
///
/// #[binread]
/// struct EmbeddedString {
///     #[br(temp)]
///     len: u8,
///     #[br(args(len.into()))]
///     string: FixedLenString
/// }
/// let mut fixed_length_string = Cursor::new(b"\x04null");
///
/// assert_eq!(
///     fixed_length_string.read_le::<EmbeddedString>().unwrap().string.to_string(),
///     "null"
/// );
///
/// ```
#[derive(Clone, Eq, PartialEq, Default)]
pub struct FixedLenString(
    /// The raw byte string.
    pub Vec<u8>,
);

impl BinRead for FixedLenString {
    type Args = (usize,);

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        
        Ok(
            Self(
                (0..args.0)
                .map(|_| u8::read_options(reader, options, ()))
                .collect::<BinResult<Vec<u8>>>()?
            )
        )

    }
}

impl BinWrite for FixedLenString {
    type Args = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &crate::WriteOptions,
        args: Self::Args,
    ) -> BinResult<()> {
        self.0.write_options(writer, options, args)?;

        Ok(())
    }
}

impl From<&str> for FixedLenString {
    fn from(s: &str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<String> for FixedLenString {
    fn from(s: String) -> Self {
        Self(s.into_bytes())
    }
}

impl From<FixedLenString> for Vec<u8> {
    fn from(s: FixedLenString) -> Self {
        s.0
    }
}

impl TryFrom<FixedLenString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: FixedLenString) -> Result<Self, Self::Error> {
        String::from_utf8(value.0)
    }
}

impl core::ops::Deref for FixedLenString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for FixedLenString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for FixedLenString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FixedLenString(\"")?;
        display_utf8(&self.0, f, str::escape_debug)?;
        write!(f, "\")")
    }
}

impl fmt::Display for FixedLenString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_utf8(&self.0, f, str::chars)
    }
}

mod test {
    #[test]
    pub fn fixed_length_string() {
        use binrw::{prelude::*, FixedLenString, io::Cursor};
        
         #[binread]
         #[derive(Debug)]
         struct EmbeddedString {
             #[br(temp)]
             _len: u8,
             #[br(args(_len.into()))]
             string: FixedLenString
         }
         let mut fixed_length_string = Cursor::new(b"\x04null");
        
         let parsed = fixed_length_string.read_ne::<EmbeddedString>();
         println!("{:?}", parsed);
         assert_eq!(
             parsed.unwrap().string.to_string(),
             "null"
         );
    }
}