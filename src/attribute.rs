//! A documentation-only module for attributes
//! 
//! # List of attributes
//! 
//! | Attribute | Supports | Description
//! |-----------|------------------|------------
//! | [big](#Byteorder) | all | Set the endianness to big endian
//! | [little](#Byteorder) | all | Set the endianness to little endian
//! | [magic](#Magic) | top-level | At the start of parsing read a value and make sure it is equivelant to a constant value
//! | [assert](#Assert) | top-level | After parsing, check if a condition is true and, optionally, return a custom error if false. Allows multiple.
//! | [import](#Arguments) | top-level | Define the arguments for parsing the given type
//! | [args](#Arguments) | fields | Pass a set of arguments.
//! 
//! # Byteorder
//! 
//! You can use `big` or `little` at either the struct-level or the field-level in order
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
//! The order of precedence is: (from highed to lowest)
//! 1. Field-level
//! 2. Variant-level (for enums)
//! 3. Top-level
//! 4. Configured (i.e. what endianess was passed in)
//! 5. Native endianess
//!
//! # Magic
//! 
//! Magic, or magic values, are constants used for sanity/integrity checking or simply for
//! making file identification easier. Since these are such a common use case binread provides
//! an attribute for handling this for you to save code/memory/time/etc.
//! 
//! The format is `magic = [lit]` where `[lit]` is any literal supported by Rust. This is allowed
//! at the following levels: struct, enum, variant, and field.
//! 
//! **Examples:**
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead, Debug)]
//! #[br(magic = b"TEST")]
//! struct Test {
//!     val: u32
//! }
//! 
//! #[derive(BinRead, Debug)]
//! #[br(magic = 1.2f32)]
//! struct Version(u16);
//! 
//! #[derive(BinRead)]
//! enum Command {
//!     #[br(magic = 0u8)] Nop,
//!     #[br(magic = 1u8)] Jump { loc: u32 },
//!     #[br(magic = 2u8)] Begin { var_count: u16, local_count: u16 }
//! }
//! ```
//! 
//! Example error:
//! ```text
//! Error::BadMagic { pos: 0x30 }
//! ```
//! See [`binread::Error`](crate::Error::BadMagic) for more info.
//!
//! # Assert
//! 
//! `assert` is the core of error handling in BinRead. It returns either an [`AssertFail`](crate::Error::AssertFail)
//! or, optionally, a custom user-generated error, allowing you to attach context from before
//! parsing failed.
//! 
//! **Custom Error Handling Example:**
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(Debug, PartialEq)]
//! struct NotSmallerError(u32, u32);
//!
//! #[derive(BinRead, Debug)]
//! #[br(assert(some_val > some_smaller_val, NotSmallerError(some_val, some_smaller_val)))]
//! struct Test {
//!     some_val: u32,
//!     some_smaller_val: u32
//! }
//!
//! let error = Cursor::new(b"\0\0\0\x01\0\0\0\xFF").read_be::<Test>();
//! assert!(error.is_err());
//! if let binread::Error::Custom { err, .. } = error.unwrap_err() {
//!     assert_eq!(
//!         err.downcast_ref::<NotSmallerError>().unwrap(),
//!         &NotSmallerError(0x1, 0xFF)
//!     );
//! }
//! ```