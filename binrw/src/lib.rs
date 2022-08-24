#![doc = include_str!("../doc/index.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(coverage_nightly, feature(no_coverage))]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

// binrw_derive expects to be able to access items in binrw via
// `::binrw::<whatever>`. Normally, this would fail in this crate
// because binrw knows of no crate called "binrw".
// This causes binrw to associate *itself* as binrw,
// meaning it makes access via ::binrw work.
#[allow(unused_extern_crates)]
extern crate self as binrw;

#[doc(hidden)]
#[cfg(feature = "std")]
pub use std as alloc;

#[doc(hidden)]
#[cfg(not(feature = "std"))]
pub extern crate alloc;

#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;

pub mod docs;
pub mod endian;
pub mod error;
pub mod file_ptr;
#[doc(hidden)]
pub mod has_magic;
pub mod helpers;
pub mod io;

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
    has_magic::HasMagic,
    helpers::{count, until, until_eof, until_exclusive},
    pos_value::PosValue,
    strings::{NullString, NullWideString},
};

/// The derive macro for [`BinRead`].
pub use binrw_derive::BinRead;

/// The attribute version of the derive macro for [`BinRead`]. Use this instead
/// of `#[derive(BinRead)]` to enable [temporary variables](docs::attribute#temp).
///
/// Note that `#[binread]` should be placed above other `#[derive(..)]` directives to avoid
/// issues where other derived methods (e.g. from `#[derive(Debug)]`) try to access fields that are
/// removed by `#[binread]`.
pub use binrw_derive::binread;

/// The derive macro for [`BinWrite`].
pub use binrw_derive::BinWrite;

/// The attribute version of the derive macro for [`BinWrite`].
pub use binrw_derive::binwrite;

/// The attribute version of the derive macro for both [`BinRead`] and [`BinWrite`]. Use
/// instead of `#[derive(BinRead, BinWrite)]` to enable [temporary variables](docs::attribute#temp).
///
/// Note that `#[binrw]` should be placed above other `#[derive(..)]` directives to avoid
/// issues where other derived methods (e.g. from `#[derive(Debug)]`) try to access fields that are
/// removed by `#[binrw]`.
pub use binrw_derive::binrw;

pub use binrw_derive::BinrwNamedArgs;

/// A specialized [`Result`] type for BinRead operations.
pub type BinResult<T> = core::result::Result<T, Error>;

mod binread;
pub use binread::*;

mod binwrite;
pub use binwrite::*;

mod builder_types;
pub use builder_types::*;

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

    pub use crate::{
        binread, binrw, binwrite, BinRead, BinReaderExt, BinResult, BinWrite, BinWriterExt,
    };
}
