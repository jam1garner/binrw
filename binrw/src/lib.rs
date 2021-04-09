//! **Maintainer's Note:** binrw is in a rather early state. Lots of work is being
//! done in order to make a great library before holding it to too many stability guarantees.
//! Expect a bit of churn, if you're looking for a stable alternative, check out [`binread`].
//! If you're interested in getting involved, join us on [discord] or [matrix]! (whichever you
//! prefer, they're bridged together)
//!
//! ---
//!
//! [`binread`]: https://docs.rs/binread
//! [discord]: https://discord.gg/ABy4Qh549j
//! [matrix]: https://matrix.to/#/#binrw:matrix.org
//!
//! binrw helps you write maintainable & easy-to-read declarative binary data
//! parsers using ✨macro magic✨.
//!
//! Adding `#[derive(BinRead)]` to any struct or enum generates a parser that
//! can read that type from raw data:
//!
//! ```
//! # use binrw::{BinRead, io::Cursor};
//! # #[derive(Debug, PartialEq)]
//! #[derive(BinRead)]
//! #[br(little)]
//! struct Point(i16, i16);
//!
//! let point = Point::read(&mut Cursor::new(b"\x80\x02\xe0\x01")).unwrap();
//! assert_eq!(point, Point(640, 480));
//!
//! # #[derive(Debug, PartialEq)]
//! #[derive(BinRead)]
//! #[br(big, magic = b"SHAP")]
//! enum Shape {
//!     #[br(magic(0u8))] Rect { left: i16, top: i16, right: i16, bottom: i16 },
//!     #[br(magic(1u8))] Oval { origin: Point, rx: u8, ry: u8 }
//! }
//!
//! let oval = Shape::read(&mut Cursor::new(b"SHAP\x01\x80\x02\xe0\x01\x2a\x15")).unwrap();
//! assert_eq!(oval, Shape::Oval { origin: Point(640, 480), rx: 42, ry: 21 });
//! ```
//!
//! Types that can’t implement `BinRead` directly (e.g. types from third party
//! crates) can also be read using
//! [free parser functions](attribute#custom-parsers) or by
//! [mapping values](attribute#map).
//!
//! Unlike “zero-copy” libraries, the in-memory representation of binrw structs
//! doesn’t need to match the raw data. This can allow for better memory
//! performance, especially on architectures where unaligned memory access is
//! slow. Also, because data is never [transmuted](core::mem::transmute), there
//! is no risk of undefined behaviour.
//!
//! # Input
//!
//! `BinRead` reads data from any object that implements [`io::Read`] +
//! [`io::Seek`]. This means that data can come from memory, network, disk, or
//! any other streaming source. It also means that low-level data operations
//! like buffering and compression are efficient and easy to implement.
//!
//! `BinRead` also includes an [extension trait](BinReaderExt) for reading types
//! directly from input objects:
//!
//! ```rust
//! use binrw::{BinReaderExt, io::Cursor};
//!
//! let mut reader = Cursor::new(b"\x00\x0A");
//! let val: u16 = reader.read_be().unwrap();
//! assert_eq!(val, 10);
//! ```
//!
//! # Directives
//!
//! Handling things like magic numbers, byte ordering, and padding & alignment
//! is typical when working with binary data, so binrw includes a variety of
//! [built-in directives](attribute) for these common cases that can be applied
//! using the `#[br]` attribute:
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor, NullString};
//! #
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
//! let mut data = Cursor::new(b"DOG\x02\x00\x01\x00\x12\0\0Rudy\0");
//! let dog = Dog::read(&mut data).unwrap();
//! assert_eq!(dog.bone_piles, &[0x1, 0x12]);
//! assert_eq!(dog.name.into_string(), "Rudy")
//! ```
//!
//! Directives can also reference earlier fields by name. For tuple types,
//! earlier fields are addressable by `self_N`, where `N` is the index of the
//! field.
//!
//! See the [attribute module](attribute) for the full list of available
//! directives.
//!
//! # Built-in implementations
//!
//! Implementations for all primitive data types, arrays, tuples, and standard
//! Rust types like [`Vec`] are included, along with parsers for other
//! frequently used binary data patterns like
//! [null-terminated strings](NullString) and
//! [indirect addressing using offsets](FilePtr). Convenient access into
//! bitfields is possible using crates like
//! [modular-bitfield](attribute#using-map-on-a-struct-to-create-a-bit-field).
//!
//! See the [`BinRead` trait](BinRead#foreign-impls) for the full list of
//! built-in implementations.
//!
//! # no_std support
//!
//! binrw supports no_std and includes a compatible subset of [`io`]
//! functionality. The [`alloc`] crate is required.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

// binrw_derive expects to be able to access items in binrw via
// `::binrw::<whatever>`. Normally, this would fail in this crate
// because binrw knows of no crate called "binrw".
// This causes binrw to associate *itself* as binrw,
// meaning it makes access via ::binrw work.
#[allow(unused_extern_crates)]
extern crate self as binrw;

#[cfg(feature = "std")]
use std as alloc;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

pub mod attribute;
pub mod endian;
pub mod error;
pub mod file_ptr;
pub mod helpers;
pub mod io;
#[doc(hidden)]
pub mod options;
#[doc(hidden)]
pub mod pos_value;
pub mod punctuated;
#[doc(hidden)]
pub mod strings;

#[doc(inline)]
pub use {
    endian::Endian,
    error::Error,
    file_ptr::{FilePtr, FilePtr128, FilePtr16, FilePtr32, FilePtr64, FilePtr8},
    options::ReadOptions,
    pos_value::PosValue,
    strings::{NullString, NullWideString},
};

/// The derive macro for [`BinRead`].
pub use binrw_derive::BinRead;

/// The attribute version of the derive macro for [`BinRead`]. Use this instead
/// of `#[derive(BinRead)]` to enable [temporary variables](attribute#temp).
pub use binrw_derive::derive_binread;

/// A specialized [`Result`] type for BinRead operations.
pub type BinResult<T> = core::result::Result<T, Error>;

mod binread_impls;
pub use binread_impls::*;

mod binread;
pub use binread::*;

pub mod prelude {
    //! The binrw prelude.
    //!
    //! A collection of traits and types you’ll likely need when working with
    //! binrw and are unlikely to cause name conflicts.
    //!
    //! ```
    //! # #![allow(unused_imports)]
    //! use binrw::prelude::*;
    //! ```

    pub use crate::BinRead;
    pub use crate::BinReaderExt;
    pub use crate::BinResult;
}
