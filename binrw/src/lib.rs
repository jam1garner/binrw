//! A Rust crate for helping parse binary data using ✨macro magic✨.
//!
//! # Example
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor, NullString};
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
//! use binrw::BinRead;
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
//! use binrw::{BinReaderExt, io::Cursor};
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
//! # use binrw::BinRead;
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
//! # use binrw::{prelude::*, io::Cursor};
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
//! # use binrw::{prelude::*, io::Cursor};
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

mod binread;
pub mod io;
pub mod error;
pub mod endian;
pub mod helpers;
pub mod file_ptr;
pub mod attribute;
pub mod punctuated;
#[doc(hidden)] pub mod strings;
#[doc(hidden)] pub mod pos_value;

#[cfg(feature = "std")]
#[cfg(feature = "debug_template")]
pub mod binary_template;

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
    binread::{
        BinRead,
        BinReaderExt,
        ReadOptions,
    },
    strings::{
        NullString,
        NullWideString
    }
};

use io::{Read, Seek, SeekFrom};

/// Derive macro for BinRead. [Usage here](BinRead).
pub use binrw_derive::BinRead;

/// Equivelant to `derive(BinRead)` but allows for temporary variables.
pub use binrw_derive::derive_binread;

mod binread_impls;
pub use binread_impls::*;

/// A Result for any binread function that can return an error
pub type BinResult<T> = core::result::Result<T, Error>;

/// The collection of traits and types you'll likely need when working with binread and are
/// unlikely to cause name conflicts.
pub mod prelude {
    pub use crate::BinRead;
    pub use crate::BinReaderExt;
    pub use crate::BinResult;
}
