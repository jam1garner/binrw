//! A documentation-only module for attributes
//! 
//! # List of attributes
//! 
//! | Attribute | Supports | Description
//! |-----------|------------------|------------
//! | [big](#byteorder) | all | Set the endianness to big endian
//! | [little](#byteorder) | all | Set the endianness to little endian
//! | [magic](#magic) | top-level | At the start of parsing read a value and make sure it is equivelant to a constant value
//! | [assert](#assert) | top-level | After parsing, check if a condition is true and, optionally, return a custom error if false. Allows multiple.
//! | [import](#arguments) | top-level | Define the arguments for parsing the given type
//! | [args](#arguments) | fields | Pass a set of arguments.
//! | [default](#default) | fields | Set a field to the default value for the type
//! | [ignore](#ignore) | fields | Ignore this field while reading
//! | [postprocess_now](#postprocessing) | fields | Immediately run `after_parse` after reading
//! | [deref_now](#postprocessing) | fields | Alias for postprocess_now
//! | [restore_position](#restore-position) | fields | Restore the reader position after reading the field
//! | [map](#map) | fields | Read a type from the reader and then apply a function to map it to the type to store in the struct
//! | [parse_with](#custom-parsers) | fields | Use a custom parser function for reading from a file
//! | [calc](#calculations) | fields | Compute an expression to store. Can use previously read values.
//! | [count](#count) | fields | Set the length for a vector
//! | [is_little](#byteorder) | fields | Conditionally set the endian to little
//! | [is_big](#byteorder) | fields | Conditionally set the endian to big
//! | [offset](#offset) | fields | Change the offset a [`FilePtr`](crate::FilePtr) is relative to
//! | [if](#condtional-values) | fields | Used on an [`Option<T>`](core::option::Option) to read a value of type `T` only if the condition is met
//! | [pad_before](#padding-and-alignment) | fields | Skip a constant number of bytes forward before reading
//! | [pad_after](#padding-and-alignment) | fields | Skip a constant number of bytes forward after reading
//! | [align_before](#padding-and-alignment) | fields | Skip to the next Nth byte before reading
//! | [align_after](#padding-and-alignment) | fields | Skip to the next Nth byte after reading
//! | [seek_before](#padding-and-alignment) | fields | Passes the given [`SeekFrom`](crate::io::SeekFrom) to [`Seek::seek`](crate::io::Seek::seek)
//! | [pad_size_to](#padding-and-alignment) | fields | Ensures the cursor is at least N bytes after the starting position for this field
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
//! binread also offers the ability to 
//!
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! 
//! #[derive(BinRead, Debug, PartialEq)]
//! #[br(big)]
//! struct MyType {
//!     val: u8,
//!     #[br(is_little = (val == 3))]
//!     other_val: u16
//! }
//! 
//! # assert_eq!(MyType::read(&mut Cursor::new(b"\x03\x01\x00")).unwrap(), MyType { val: 3, other_val: 1 });
//! ```
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
//! let error = error.unwrap_err();
//! assert_eq!(error.custom_err(), Some(&NotSmallerError(0x1, 0xFF)));
//! ```
//! 
//! # Arguments
//! One feature of binread is allowing arguments to be passed to the type in order to tell
//! the type any info it needs to parse the data. To accept arguments when using the derive
//! macro, you can use the `import` attribute and to pass arguments you can use the `args`
//! attribute.
//! 
//! **Example:**
//! ```rust
//! # use binread::prelude::*;
//! 
//! #[derive(BinRead)]
//! #[br(import(val1: u32, val2: &'static str))]
//! struct ImportTest {
//!     // ...
//! }
//! 
//! #[derive(BinRead)]
//! struct ArgsTets {
//!     val: u32,
//!     #[br(args(val + 3, "test"))]
//!     test: ImportTest
//! }
//! ```
//! 