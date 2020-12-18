//! A Rust crate for helping parse binary data using ✨macro magic✨.
//!
//! # Example
//!
//! ```
//! # use binread::{prelude::*, io::Cursor, NullString};
//!
//! #[derive(BinRead)]
//! #[br(magic = b"DOG", assert(name.len() != 0))]
//! struct Dog {
//!     bone_pile_count: u8,
//!
//!     #[br(big, count = bone_pile_count)]
//!     bone_piles: Vec<u16>,
//!
//!     #[br(align_before = 0xA)]
//!     name: NullString
//! }
//!
//! let mut reader = Cursor::new(b"DOG\x02\x00\x01\x00\x12\0\0Rudy\0");
//! let dog: Dog = reader.read_ne().unwrap();
//! assert_eq!(dog.bone_piles, &[0x1, 0x12]);
//! assert_eq!(dog.name.into_string(), "Rudy")
//! ```
//!
//! # The Basics
//!
//! At the core of `binread` is the [`BinRead`](BinRead) trait. It defines how to read
//! a type from bytes and is already implemented for most primitives and simple collections.
//!
//! ```rust
//! use binread::BinRead;
//! use std::io::Cursor;
//!
//! let mut reader = Cursor::new(b"\0\0\0\x01");
//! let val = u32::read(&mut reader).unwrap();
//! ```
//!
//! However, [`read`](BinRead::read) is intentionally simple and, as a result, doesn't even
//! allow you to configure the byte order. For that you need [`read_options`](BinRead::read_options)
//! which, while more powerful, isn't exactly ergonomics.
//!
//! So, as a balance between ergonomics and configurability you have the [`BinReaderExt`](BinReaderExt)
//! trait. It is an extension for readers to allow for you to directly read any BinRead types from
//! any reader.
//!
//! Example:
//! ```rust
//! use binread::{BinReaderExt, io::Cursor};
//!
//! let mut reader = Cursor::new(b"\x00\x0A");
//! let val: u16 = reader.read_be().unwrap();
//! assert_eq!(val, 10);
//! ```
//!
//! It even works for tuples and arrays of BinRead types for up to size 32.
//!
//! # Derive Macro
//!
//! The most significant feature of binread is its ability to use the Derive macro to
//! implement [`BinRead`](BinRead) for your own types. This allows you to replace repetitive
//! imperative code with declarative struct definitions for your binary data parsing.
//!
//! ## Basic Derive Example
//! ```rust
//! # use binread::BinRead;
//! #[derive(BinRead)]
//! struct MyType {
//!     first: u32,
//!     second: u32
//! }
//!
//! // Also works with tuple types!
//! #[derive(BinRead)]
//! struct MyType2(u32, u32);
//! ```
//! ## Attributes
//! The BinRead derive macro uses attributes in order to allow for more complicated parsers. For
//! example you can use `big` or `little` at either the struct-level or the field-level in order
//! to override the byte order of values.
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! #[br(little)]
//! struct MyType (
//!     #[br(big)] u32, // will be big endian
//!     u32, // will be little endian
//! );
//! ```
//! The order of precedence is: (from highest to lowest)
//! 1. Field-level
//! 2. Variant-level (for enums)
//! 3. Top-level
//! 4. Configured (i.e. what endianess was passed in)
//! 5. Native endianess
//!
//! For a list of attributes see the [`attribute`](attribute) module
//!
//! ## Generics
//! The BinRead derive macro also allows for generic parsing. That way you can build up
//! higher-level parsers that can have their type swapped out to allow greater reuse of code.
//!
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct U32CountVec<T: BinRead<Args=()>> {
//!     count: u32,
//!     #[br(count = count)]
//!     data: Vec<T>,
//! }
//! ```
//!
//! In order to parse generically, we have to (in some way) bound `Args`. The easiest way to do
//! this is to bound `<T as BinRead>::Args` to `()` (no arguments), however it is also possible to
//! either accept a specific set of arguments or be generic over the given arguments.
#![cfg_attr(not(feature="std"), no_std)]

#[cfg(feature = "std")]
use std as alloc;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    vec::Vec,
    string::String,
};

pub mod io;
pub mod error;
pub mod endian;
pub mod helpers;
pub mod file_ptr;
pub mod attribute;
pub mod punctuated;
#[doc(hidden)] pub mod options;
#[doc(hidden)] pub mod strings;
#[doc(hidden)] pub mod pos_value;

#[cfg(feature = "std")]
#[cfg(feature = "debug_template")]
pub mod binary_template;

use core::any::{Any, TypeId};

#[doc(inline)]
pub use {
    error::Error,
    endian::Endian,
    pos_value::PosValue,
    file_ptr::{
        FilePtr,
        FilePtr8,
        FilePtr16,
        FilePtr32,
        FilePtr64,
        FilePtr128,
    },
    options::ReadOptions,
    strings::{
        NullString,
        NullWideString
    }
};

use io::{Read, Seek, SeekFrom};

/// Derive macro for BinRead. [Usage here](BinRead).
pub use binread_derive::BinRead;

/// Equivelant to `derive(BinRead)` but allows for temporary variables.
pub use binread_derive::derive_binread;

mod binread_impls;
pub use binread_impls::*;

/// A Result for any binread function that can return an error
pub type BinResult<T> = core::result::Result<T, Error>;

/// A `BinRead` trait allows reading a structure from anything that implements [`io::Read`](io::Read) and [`io::Seek`](io::Seek)
/// BinRead is implemented on the type to be read out of the given reader
pub trait BinRead: Sized {
    /// The type of arguments needed to be supplied in order to read this type, usually a tuple.
    ///
    /// **NOTE:** For types that don't require any arguments, use the unit (`()`) type. This will allow [`read`](BinRead::read) to be used.
    type Args: Any + Copy;

    /// Read the type from the reader while assuming no arguments have been passed
    ///
    /// # Panics
    /// Panics if there is no [`args_default`](BinRead::args_default) implementation
    fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        let args = match Self::args_default() {
            Some(args) => args,
            None => panic!("Must pass args, no args_default implemented")
        };

        Self::read_options(reader, &ReadOptions::default(), args)
    }

    /// Read the type from the reader using the specified arguments
    fn read_args<R: Read + Seek>(reader: &mut R, args: Self::Args) -> BinResult<Self> {
        Self::read_options(reader, &ReadOptions::default(), args)
    }

    /// Read the type from the reader
    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self>;

    fn after_parse<R: Read + Seek>(&mut self, _: &mut R, _: &ReadOptions, _: Self::Args) -> BinResult<()> {
        Ok(())
    }

    /// The default arguments to be used when using the [`read`](BinRead::read) shortcut method.
    /// Override this for any type that optionally requries arguments
    fn args_default() -> Option<Self::Args> {
        // Trick to effectively get specialization on stable, should constant-folded away
        // Returns `Some(())` if Self::Args == (), otherwise returns `None`
        if TypeId::of::<Self::Args>() == TypeId::of::<()>() {
            Some(unsafe{
                core::mem::MaybeUninit::uninit().assume_init()
            })
        } else {
            None
        }
    }
}

/// An extension trait for [`io::Read`](io::Read) to provide methods for reading a value directly
///
/// ## Example
/// ```rust
/// use binread::prelude::*; // BinReadExt is in the prelude
/// use binread::endian::LE;
/// use std::io::Cursor;
///
/// let mut reader = Cursor::new(b"\x07\0\0\0\xCC\0\0\x05");
/// let x: u32 = reader.read_le().unwrap();
/// let y: u16 = reader.read_type(LE).unwrap();
/// let z = reader.read_be::<u16>().unwrap();
///
/// assert_eq!((x, y, z), (7u32, 0xCCu16, 5u16));
/// ```
pub trait BinReaderExt: Read + Seek + Sized {
    /// Read the given type from the reader using the given endianness.
    fn read_type<T: BinRead>(&mut self, endian: Endian) -> BinResult<T> {
        let args = match T::args_default() {
            Some(args) => args,
            None => panic!("Must pass args, no args_default implemented")
        };

        let options = ReadOptions{
            endian, ..Default::default()
        };

        let mut res = T::read_options(self, &options, args)?;
        res.after_parse(self, &options, args)?;

        Ok(res)
    }

    /// Read the given type from the reader with big endian byteorder
    fn read_be<T: BinRead>(&mut self) -> BinResult<T> {
        self.read_type(Endian::Big)
    }

    /// Read the given type from the reader with little endian byteorder
    fn read_le<T: BinRead>(&mut self) -> BinResult<T> {
        self.read_type(Endian::Little)
    }

    /// Read the given type from the reader with the native byteorder
    fn read_ne<T: BinRead>(&mut self) -> BinResult<T> {
        self.read_type(Endian::Native)
    }
}

impl<R: Read + Seek + Sized> BinReaderExt for R {}

/// The collection of traits and types you'll likely need when working with binread and are
/// unlikely to cause name conflicts.
pub mod prelude {
    pub use crate::BinRead;
    pub use crate::BinReaderExt;
    pub use crate::BinResult;
}
