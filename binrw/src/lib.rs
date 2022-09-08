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

/// Attribute macro used to generate
/// [`parse_with`](docs::attribute#custom-parserswriters) functions.
///
/// Rust functions are transformed by this macro to match the binrw API.
///
/// # Attribute options
///
/// * `#[parser(reader)]` or `#[parser(reader: $ident)]`: Exposes the write
///   stream to the function. If no variable name is given, `reader` is used.
/// * `#[parser(endian)]` or `#[parser(endian: $ident)]`: Exposes the endianness
///   to the function. If no variable name is given, `endian` is used.
///
/// Options are comma-separated.
///
/// # Function parameters
///
/// Parameters are transformed into either
/// [tuple-style arguments](docs::attribute#tuple-style-arguments) or
/// [raw arguments](docs::attribute#raw-arguments) depending upon the function
/// signature.
///
/// ## Tuple-style arguments
///
/// Use a normal function signature. The parameters in the signature will be
/// converted to a tuple. For example:
///
/// ```
/// #[binrw::parser(reader: r, endian)]
/// fn custom_parser(v0: u8, v1: i16) -> binrw::BinResult<()> {
///     Ok(())
/// }
/// # custom_parser(&mut binrw::io::Cursor::new(b""), binrw::Endian::Little, (0, 0)).unwrap();
/// ```
///
/// The transformed output for this function is:
///
/// ```
/// use binrw::{BinResult, Endian, io::{Read, Seek}};
/// fn custom_parser<R: Read + Seek>(
///     r: &mut R,
///     endian: Endian,
///     (v0, v1): (u8, i16)
/// ) -> BinResult<()> {
///     Ok(())
/// }
/// # custom_parser(&mut binrw::io::Cursor::new(b""), binrw::Endian::Little, (0, 0)).unwrap();
/// ```
///
/// ## Raw arguments
///
/// Use a *variadic* function signature with a single parameter. The name and
/// type of the parameter will be used as the raw argument. For example:
///
/// ```
/// # struct ArgsType;
/// #[binrw::parser]
/// fn custom_parser(args: ArgsType, ...) -> binrw::BinResult<()> {
///     Ok(())
/// }
/// # custom_parser(&mut binrw::io::Cursor::new(b""), binrw::Endian::Little, ArgsType).unwrap();
/// ```
///
/// The transformed output for this function is:
///
/// ```
/// # struct ArgsType;
/// use binrw::{BinResult, Endian, io::{Read, Seek}};
/// fn custom_parser<R: Read + Seek>(
///     _: &mut R,
///     _: Endian,
///     args: ArgsType
/// ) -> BinResult<()> {
///     Ok(())
/// }
/// # custom_parser(&mut binrw::io::Cursor::new(b""), binrw::Endian::Little, ArgsType).unwrap();
/// ```
///
/// # Return value
///
/// The return value of a parser function must be [`BinResult<T>`](BinResult),
/// where `T` is the type of the object being parsed.
pub use binrw_derive::parser;

/// Attribute macro used to generate
/// [`write_with`](docs::attribute#custom-parserswriters) functions.
///
/// Rust functions are transformed by this macro to match the binrw API.
///
/// # Attribute options
///
/// * `#[writer(writer)]` or `#[writer(writer: $ident)]`: Exposes the write
///   stream to the function. If no variable name is given, `writer` is used.
/// * `#[writer(endian)]` or `#[writer(endian: $ident)]`: Exposes the endianness
///   to the function. If no variable name is given, `endian` is used.
///
/// Options are comma-separated.
///
/// # Function parameters
///
/// The first parameter is required and receives a reference to the object being
/// written.
///
/// Subsequent parameters are transformed into either
/// [tuple-style arguments](docs::attribute#tuple-style-arguments) or
/// [raw arguments](docs::attribute#raw-arguments) depending upon the function
/// signature.
///
/// ## Tuple-style arguments
///
/// Use a normal function signature. The remaining parameters in the signature
/// will be converted to a tuple. For example:
///
/// ```
/// # struct Object;
/// #[binrw::writer(writer: w, endian)]
/// fn custom_writer(obj: &Object, v0: u8, v1: i16) -> binrw::BinResult<()> {
///     Ok(())
/// }
/// # custom_writer(&Object, &mut binrw::io::Cursor::new(vec![]), binrw::Endian::Little, (0, 0)).unwrap();
/// ```
///
/// The transformed output for this function is:
///
/// ```
/// # struct Object;
/// use binrw::{BinResult, Endian, io::{Seek, Write}};
/// fn custom_writer<W: Write + Seek>(
///     obj: &Object,
///     w: &mut W,
///     endian: Endian,
///     (v0, v1): (u8, i16)
/// ) -> BinResult<()> {
///     Ok(())
/// }
/// # custom_writer(&Object, &mut binrw::io::Cursor::new(vec![]), binrw::Endian::Little, (0, 0)).unwrap();
/// ```
///
/// ## Raw arguments
///
/// Use a *variadic* function signature with a second parameter. The name and
/// type of the second parameter will be used as the raw argument. For example:
///
/// ```
/// # struct Object;
/// # struct ArgsType;
/// #[binrw::writer]
/// fn custom_writer(obj: &Object, args: ArgsType, ...) -> binrw::BinResult<()> {
///     Ok(())
/// }
/// # custom_writer(&Object, &mut binrw::io::Cursor::new(vec![]), binrw::Endian::Little, ArgsType).unwrap();
/// ```
///
/// The transformed output for this function is:
///
/// ```
/// # struct Object;
/// # struct ArgsType;
/// use binrw::{BinResult, Endian, io::{Seek, Write}};
/// fn custom_writer<W: Write + Seek>(
///     obj: &Object,
///     _: &mut W,
///     _: Endian,
///     args: ArgsType
/// ) -> BinResult<()> {
///     Ok(())
/// }
/// # custom_writer(&Object, &mut binrw::io::Cursor::new(vec![]), binrw::Endian::Little, ArgsType).unwrap();
/// ```
///
/// # Return value
///
/// The return value of a writer function must be [`BinResult<()>`](BinResult).
pub use binrw_derive::writer;

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
