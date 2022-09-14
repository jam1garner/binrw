#![doc = include_str!("../doc/index.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(coverage_nightly, feature(no_coverage))]
#![cfg_attr(all(doc, nightly), feature(doc_cfg))]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
// Lint: This is not beneficial for code organisation.
#![allow(clippy::module_name_repetitions)]

extern crate alloc;
// This extern crate declaration is required to use binrw_derive macros like
// BinrwNamedArgs inside binrw because the generated code references a binrw
// crate, but binrw is not a dependency of binrw so no crate with that name gets
// automatically added by cargo to the extern prelude.
extern crate self as binrw;
#[cfg(all(doc, not(feature = "std")))]
extern crate std;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;
mod binread;
mod binwrite;
mod builder_types;
pub mod docs;
pub mod endian;
pub mod error;
pub mod file_ptr;
pub mod helpers;
pub mod io;
pub mod meta;
#[doc(hidden)]
pub mod pos_value;
pub mod punctuated;
#[doc(hidden)]
pub mod strings;

#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;
#[doc(inline)]
pub use {
    binread::*,
    binwrite::*,
    builder_types::*,
    endian::Endian,
    error::Error,
    file_ptr::{FilePtr, FilePtr128, FilePtr16, FilePtr32, FilePtr64, FilePtr8},
    helpers::{count, until, until_eof, until_exclusive},
    pos_value::PosValue,
    strings::{NullString, NullWideString},
};

/// The derive macro for [`BinRead`].
pub use binrw_derive::BinRead;

/// The attribute version of the derive macro for [`BinRead`]. Use this instead
/// of `#[derive(BinRead)]` to enable [temporary variables](docs::attribute#temp).
///
/// Note that `#[binread]` should be placed above other `#[derive(..)]`
/// directives to avoid issues where other derived methods (e.g.
/// from `#[derive(Debug)]`) try to access fields that are removed by
/// `#[binread]`.
pub use binrw_derive::binread;

/// The derive macro for [`BinWrite`].
pub use binrw_derive::BinWrite;

/// The attribute version of the derive macro for [`BinWrite`]. Use this instead
/// of `#[derive(BinWrite)]` to enable
/// [temporary variables](docs::attribute#temp).
///
/// Note that `#[binwrite]` should be placed above other `#[derive(..)]`
/// directives to avoid issues where other derived methods (e.g. from
/// `#[derive(Debug)]`) try to access fields that are removed by `#[binwrite]`.
pub use binrw_derive::binwrite;

/// The attribute version of the derive macro for both [`BinRead`] and
/// [`BinWrite`]. Use this instead of `#[derive(BinRead, BinWrite)]` to enable
/// [temporary variables](docs::attribute#temp).
///
/// Note that `#[binrw]` should be placed above other `#[derive(..)]` directives
/// to avoid issues where other derived methods (e.g. from `#[derive(Debug)]`)
/// try to access fields that are removed by `#[binrw]`.
pub use binrw_derive::binrw;

/// The derive macro for [`BinrwNamedArgs`].
pub use binrw_derive::BinrwNamedArgs;

/// A specialized [`Result`] type for binrw operations.
pub type BinResult<T> = core::result::Result<T, Error>;

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
