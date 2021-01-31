use super::*;

// #[cfg(feature = "std")]
// use std::{
//     ffi::CString,
// };

use core::num::{NonZeroU8, NonZeroU16};

/*
#[cfg(feature = "std")]
impl BinRead for CString {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self>
    {
        <Vec<NonZeroU8>>::read_options(reader, options, args)
            .map(|bytes| bytes.into())
    }
}*/

impl BinRead for Vec<NonZeroU8> {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, _: &ReadOptions, _: Self::Args) -> BinResult<Self>
    {
        reader
            .iter_bytes()
            .take_while(|x| !matches!(x, Ok(0)))
            .map(|x| Ok(x.map(|byte| unsafe { NonZeroU8::new_unchecked(byte) })?))
            .collect()
    }
}

/// A null terminated UTF-8 string designed to make reading any null-terminated data easier.
///
/// **Note:** Does not include the null.
#[derive(Clone, PartialEq, Default)]
pub struct NullString(pub Vec<u8>);

/// A null terminated UTF-16 string designed to make reading any 16 bit wide null-terminated data easier.
///
/// **Note:** Does not include the null.
///
/// **Note:** This is endian dependent on a per-character basis. Will read `u16`s until a `0u16` is found.
#[derive(Clone, PartialEq, Default)]
pub struct NullWideString(pub Vec<u16>);

impl NullString {
    pub fn into_string(self) -> String {
        String::from_utf8_lossy(&self.0).into()
    }

    pub fn into_string_lossless(self) -> Result<String, alloc::string::FromUtf8Error> {
        String::from_utf8(self.0)
    }
}

impl NullWideString {
    pub fn into_string(self) -> String {
        String::from_utf16_lossy(&self.0)
    }

    pub fn into_string_lossless(self) -> Result<String, alloc::string::FromUtf16Error> {
        String::from_utf16(&self.0)
    }
}

impl Into<NullWideString> for Vec<NonZeroU16> {
    fn into(self) -> NullWideString {
        let vals: Vec<u16> = self.into_iter().map(|x| x.get()).collect();
        NullWideString(vals)
    }
}

impl Into<NullString> for Vec<NonZeroU8> {
    fn into(self) -> NullString {
        let vals: Vec<u8> = self.into_iter().map(|x| x.get()).collect();
        NullString(vals)
    }
}

impl Into<Vec<u16>> for NullWideString {
    fn into(self) -> Vec<u16> {
        self.0
    }
}

impl Into<Vec<u8>> for NullString {
    fn into(self) -> Vec<u8> {
        self.0
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
        #[cfg(feature = "debug_template")]
        let options = {
            let mut options = *options;
            let pos = reader.seek(SeekFrom::Current(0)).unwrap();

            if !options.dont_output_to_template {
                binary_template::write_named(
                    options.endian,
                    pos,
                    "wstring",
                    &options.variable_name
                        .map(ToString::to_string)
                        .unwrap_or_else(binary_template::get_next_var_name)
                );

            }
            options.dont_output_to_template = true;
            options
        };

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
        #[cfg(feature = "debug_template")] {
            let pos = reader.seek(SeekFrom::Current(0)).unwrap();

            if !options.dont_output_to_template {
                binary_template::write_named(
                    options.endian,
                    pos,
                    "string",
                    &options.variable_name
                            .map(ToString::to_string)
                            .unwrap_or_else(binary_template::get_next_var_name)
                );
            }
        }

        // https://github.com/rust-lang/rust-clippy/issues/6447
        #[allow(clippy::unit_arg)]
        <Vec<NonZeroU8>>::read_options(reader, options, args)
            .map(|chars| chars.into())
    }
}

use core::fmt;

impl fmt::Debug for NullString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NullString({:?})", self.clone().into_string())
    }
}

impl fmt::Debug for NullWideString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NullWideString({:?})", self.clone().into_string())
    }
}

impl std::ops::Deref for NullString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for NullWideString {
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