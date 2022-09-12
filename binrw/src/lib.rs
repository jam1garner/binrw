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
// NamedArgs inside binrw because the generated code references a binrw crate,
// but binrw is not a dependency of binrw so no crate with that name gets
// automatically added by cargo to the extern prelude.
extern crate self as binrw;
#[cfg(all(doc, not(feature = "std")))]
extern crate std;

#[doc(hidden)]
#[path = "private.rs"]
pub mod __private;
mod binread;
mod binwrite;
pub mod docs;
pub mod endian;
pub mod error;
pub mod file_ptr;
pub mod helpers;
pub mod io;
pub mod meta;
mod named_args;
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
    endian::Endian,
    error::Error,
    file_ptr::{FilePtr, FilePtr128, FilePtr16, FilePtr32, FilePtr64, FilePtr8},
    helpers::{count, until, until_eof, until_exclusive},
    named_args::*,
    pos_value::PosValue,
    strings::{NullString, NullWideString},
};

/// Derive macro generating an impl of the trait [`BinRead`].
///
/// See the [directives glossary](docs::attribute) for usage details.
pub use binrw_derive::BinRead;

/// Attribute macro used to generate an impl of the trait [`BinRead`] with
/// support for [temporary variables](docs::attribute#temp).
///
/// When using temporary variables, this attribute **must** be placed above
/// other attributes that generate code (e.g. `#[derive(Debug)]`) to ensure that
/// the deleted temporary fields aren’t visible to those macros.
///
/// See the [directives glossary](docs::attribute) for usage details.
pub use binrw_derive::binread;

/// Derive macro generating an impl of the trait [`BinWrite`].
///
/// See the [directives glossary](docs::attribute) for usage details.
pub use binrw_derive::BinWrite;

/// Attribute macro used to generate an impl of the trait [`BinWrite`] with
/// support for [temporary variables](docs::attribute#temp).
///
/// When using temporary variables, this attribute **must** be placed above
/// other attributes that generate code (e.g. `#[derive(Debug)]`) to ensure that
/// the deleted temporary fields aren’t visible to those macros.
///
/// See the [directives glossary](docs::attribute) for usage details.
pub use binrw_derive::binwrite;

/// Attribute macro used to generate an impl of both [`BinRead`] and
/// [`BinWrite`] traits with support for
/// [temporary variables](docs::attribute#temp).
///
/// When using temporary variables, this attribute **must** be placed above
/// other attributes that generate code (e.g. `#[derive(Debug)]`) to ensure that
/// the deleted temporary fields aren’t visible to those macros.
///
/// See the [directives glossary](docs::attribute) for usage details.
pub use binrw_derive::binrw;

/// Derive macro generating an impl of the trait [`NamedArgs`].
///
/// The use cases for this macro are:
///
/// 1. When manually implementing [`BinRead`] or [`BinWrite`] on a type where
///    named arguments are desired.
/// 2. When creating a
///    [custom parser or writer](docs::attribute#custom-parserswriters)
///    where named arguments are desired.
/// 3. When a named arguments type should be shared by several different types
///    (e.g. by using [`import_raw`](docs::attribute#raw-arguments) on
///    derived types, and by assigning the type to [`BinRead::Args`] or
///    [`BinWrite::Args`] in manual implementations).
///
/// # Field options
///
/// * `#[named_args(default = $expr)]`: Sets the default value for a field.
///
/// # Examples
///
/// ```
/// use binrw::{args, binread, BinRead, NamedArgs};
/// #[derive(Clone, NamedArgs)]
/// struct GlobalArgs<Inner> {
///     #[named_args(default = 1)]
///     version: i16,
///     inner: Inner,
/// }
///
/// #[binread]
/// #[br(import_raw(args: GlobalArgs<T::Args>))]
/// struct Container<T: BinRead> {
///     #[br(temp, if(args.version > 1, 16))]
///     count: u16,
///     #[br(args {
///         count: count.into(),
///         inner: args.inner
///     })]
///     items: Vec<T>,
/// }
///
/// # let mut input = binrw::io::Cursor::new(b"\x02\0\x42\0\x69\0");
/// # assert_eq!(
/// #     Container::<u16>::read_le_args(&mut input, args! { version: 2, inner: () }).unwrap().items,
/// #     vec![0x42, 0x69]
/// # );
/// ```
pub use binrw_derive::NamedArgs;

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
