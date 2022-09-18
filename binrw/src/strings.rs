//! Type definitions for string readers.

use crate::{
    helpers::until_exclusive,
    io::{Read, Seek, Write},
    meta::{EndianKind, ReadEndian, WriteEndian},
    BinResult, BinWrite, Endian, ReadFrom, WriteInto,
};
use alloc::{boxed::Box, string::String, vec::Vec};

/// A converter for null-terminated 8-bit strings.
///
/// The null terminator is consumed and not included in the value.
///
/// ```
/// use binrw::{BinRead, BinReaderExt, NullString, io::Cursor};
///
/// let mut null_separated_strings = Cursor::new(b"null terminated strings? in my system's language?\0no thanks\0");
///
/// assert_eq!(
///     null_separated_strings.read_with::<NullString, String>().unwrap(),
///     "null terminated strings? in my system's language?"
/// );
///
/// assert_eq!(
///     null_separated_strings.read_with::<NullString, Vec<u8>>().unwrap(),
///     b"no thanks"
/// );
/// ```
pub enum NullString {}

impl ReadEndian for NullString {
    const ENDIAN: EndianKind = EndianKind::None;
}

impl ReadFrom<NullString> for String {
    type Args = ();

    fn read_from<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<Self> {
        let pos = reader.stream_position()?;
        <Vec<u8> as ReadFrom<NullString>>::read_from(reader, endian, args).and_then(|vec| {
            Self::from_utf8(vec).map_err(|err| binrw::Error::Custom {
                pos,
                err: Box::new(err) as _,
            })
        })
    }
}

impl ReadFrom<NullString> for Vec<u8> {
    type Args = ();

    fn read_from<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<Self> {
        until_exclusive(|b| *b == 0)(reader, endian, args)
    }
}

impl WriteEndian for NullString {
    const ENDIAN: EndianKind = EndianKind::None;
}

impl WriteInto<NullString> for String {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        <_ as WriteInto<NullString>>::write_into(self.as_bytes(), writer, endian, args)
    }
}

impl WriteInto<NullString> for str {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        <_ as WriteInto<NullString>>::write_into(self.as_bytes(), writer, endian, args)
    }
}

impl WriteInto<NullString> for Vec<u8> {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        <_ as WriteInto<NullString>>::write_into(self.as_slice(), writer, endian, args)
    }
}

impl WriteInto<NullString> for [u8] {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        self.write_options(writer, endian, args)?;
        0_u8.write_options(writer, endian, args)
    }
}

impl<const N: usize> WriteInto<NullString> for [u8; N] {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        <_ as WriteInto<NullString>>::write_into(self.as_slice(), writer, endian, args)
    }
}

/// A converter for null-terminated 16-bit strings.
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
///     wide_strings.read_le_with::<NullWideString, String>().unwrap(),
///     "wide strings"
/// );
///
/// assert_eq!(
///     // notice: read_be
///     are_endian_dependent.read_be_with::<NullWideString, String>().unwrap(),
///     "are endian dependent"
/// );
/// ```
pub enum NullWideString {}

impl ReadFrom<NullWideString> for Vec<u16> {
    type Args = ();

    fn read_from<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<Self> {
        until_exclusive(|b| *b == 0)(reader, endian, args)
    }
}

impl ReadFrom<NullWideString> for String {
    type Args = ();

    fn read_from<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<Self> {
        let pos = reader.stream_position()?;
        <Vec<u16> as ReadFrom<NullWideString>>::read_from(reader, endian, args).and_then(|vec| {
            String::from_utf16(&vec).map_err(|err| binrw::Error::Custom {
                pos,
                err: Box::new(err) as _,
            })
        })
    }
}

impl WriteInto<NullWideString> for String {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        <_ as WriteInto<NullWideString>>::write_into(self.as_str(), writer, endian, args)
    }
}

impl WriteInto<NullWideString> for str {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        for c in self.encode_utf16() {
            c.write_options(writer, endian, ())?;
        }
        0_u16.write_options(writer, endian, args)
    }
}

impl WriteInto<NullWideString> for Vec<u16> {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        <_ as WriteInto<NullWideString>>::write_into(self.as_slice(), writer, endian, args)
    }
}

impl WriteInto<NullWideString> for [u16] {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        self.write_options(writer, endian, args)?;
        0_u16.write_options(writer, endian, args)
    }
}

impl<const N: usize> WriteInto<NullWideString> for [u16; N] {
    type Args = ();

    fn write_into<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args,
    ) -> BinResult<()> {
        <_ as WriteInto<NullWideString>>::write_into(self.as_slice(), writer, endian, args)
    }
}
