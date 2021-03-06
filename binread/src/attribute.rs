//! A documentation-only module for the possible directives used in `#[br]` and
//! `#[binread]` attributes.
//!
//! # List of directives
//!
//! | Directive | Supports | Description
//! |-----------|----------|------------
//! | [`align_after`](#padding-and-alignment) | field | Aligns the reader to the Nth byte after reading data.
//! | [`align_before`](#padding-and-alignment) | field | Aligns the reader to the Nth byte before reading data.
//! | [`args`](#arguments) | struct field, data variant | Passes arguments to another `BinRead` object.
//! | [`args_tuple`](#arguments) | struct field, data variant | Like `args`, but specifies a tuple containing the arguments.
//! | [`assert`](#assert) | struct, field, non-unit enum, data variant | Asserts that a condition is true. Can be used multiple times.
//! | [`big`](#byte-order) | all except unit variant | Sets the byte order to big-endian.
//! | [`calc`](#calculations) | field | Computes the value of a field instead of reading data.
//! | [`count`](#count) | field | Sets the length of a vector.
//! | [`default`](#default) | field | Uses the [`default`](core::default::Default) value for a field instead of reading data.
//! | [`deref_now`](#postprocessing) | field | An alias for `postprocess_now`.
//! | [`if`](#conditional-values) | field | Reads data only if a condition is true.
//! | [`ignore`](#default) | field | An alias for `default`.
//! | [`import`](#arguments) | struct, non-unit enum, unit-like enum | Defines extra arguments for a struct or enum.
//! | [`import_tuple`](#arguments) | struct, non-unit enum, unit-like enum | Like `import`, but receives the arguments as a tuple.
//! | [`is_big`](#byte-order) | field | Conditionally sets the byte order to big-endian.
//! | [`is_little`](#byte-order) | field | Conditionally set the byte order to little-endian.
//! | [`little`](#byte-order) | all except unit variant | Sets the byte order to little-endian.
//! | [`magic`](#magic) | all | Matches a magic number.
//! | [`map`](#map) | all except unit variant | Maps a read value to a new value. When used on a struct or enum, the map function must return `Self`.
//! | [`offset`](#offset) | field | Modifies the offset used by a [`FilePtr`](crate::FilePtr).
//! | [`pad_after`](#padding-and-alignment) | field | Skips N bytes after reading a field.
//! | [`pad_before`](#padding-and-alignment) | field | Skips N bytes before reading a field.
//! | [`pad_size_to`](#padding-and-alignment) | field | Ensures the reader is at least N bytes after the starting position for this field.
//! | [`parse_with`](#custom-parsers) | field | Specifies a custom function for reading a field.
//! | [`postprocess_now`](#postprocessing) | field | Calls [`after_parse`](crate::BinRead::after_parse) immediately after reading data instead of after all fields have been read.
//! | [`pre_assert`](#pre-assert) | struct, non-unit enum, unit variant | Like `assert`, but checks the condition before parsing.
//! | [`repr`](#repr) | unit-like enum | Specifies the underlying type for a unit-like (C-style) enum.
//! | [`restore_position`](#restore-position) | field | Restores the reader’s position after reading a field.
//! | [`return_all_errors`](#enum-errors) | non-unit enum | Returns a [`Vec`] containing the error which occurred on each variant of an enum on failure. This is the default.
//! | [`return_unexpected_error`](#enum-errors) | non-unit enum | Returns a single generic error on failure.
//! | [`seek_before`](#padding-and-alignment) | field | Moves the reader to a specific position before reading data.
//! | [`temp`](#temp) | field | Uses a field as a temporary variable. Only usable with the [`derive_binread`] attribute macro.
//! | [`try`](#try) | field | Reads data into an [`Option`](core::option::Option), but stores `None` if parsing fails instead of returning an error.
//! | [`try_map`](#map) | all except unit variant | Like `map`, but returns a [`BinResult`](crate::BinResult).
//!
//! # Byte order
//!
//! The `big` and `little` directives specify the [byte order](https://en.wikipedia.org/wiki/Endianness)
//! of data in a struct, enum, variant, or field:
//!
//! ```text
//! #[br(big)]
//! #[br(little)]
//! ```
//!
//! The `is_big` and `is_little` directives conditionally set the byte order of
//! a struct field:
//!
//! ```text
//! #[br(is_little = $cond:expr)] or #[br(is_little($cond:expr))]
//! #[br(is_big = $cond:expr)] or #[br(is_big($cond:expr))]
//! ```
//!
//! The `is_big` and `is_little` directives are primarily useful when byte order
//! is defined in the data itself. Any earlier field or [import](#arguments) can
//! be referenced in the condition. Conditional byte order directives can only
//! be used on struct fields.
//!
//! The order of precedence (from highest to lowest) for determining byte order
//! within an object is:
//!
//! 1. A directive on a field
//! 2. A directive on an enum variant
//! 3. A directive on the struct or enum
//! 4. The [`endian`](crate::ReadOptions::endian) property of the
//!    [`ReadOptions`](crate::ReadOptions) object passed to
//!    [`BinRead::read_options`](crate::BinRead::read_options) by the caller
//! 5. The host machine’s native byte order
//!
//! However, if a byte order directive is added to a struct or enum, that byte
//! order will *always* be used, even if the object is embedded in another
//! object or explicitly called with a different byte order:
//!
//! ```
//! # use binread::{Endian, ReadOptions, prelude::*, io::Cursor};
//! #[derive(BinRead, Debug, PartialEq)]
//! #[br(little)] // ← this *forces* the struct to be little-endian
//! struct Child(u32);
//!
//! #[derive(BinRead, Debug)]
//! struct Parent {
//!     #[br(big)] // ← this will be ignored
//!     child: Child,
//! };
//!
//! let mut options = ReadOptions::default();
//! options.endian = Endian::Big; // ← this will be ignored
//! # assert_eq!(
//! Child::read_options(&mut Cursor::new(b"\x01\0\0\0"), &options, ())
//! # .unwrap(), Child(1));
//! ```
//!
//! When manually implementing
//! [`BinRead::read_options`](crate::BinRead::read_options) or a
//! [custom parser function](#custom-parsers), the byte order is accessible
//! from [`ReadOptions::endian`](crate::ReadOptions::endian).
//!
//! ## Examples
//!
//! ```
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! #[br(little)]
//! struct MyType (
//!     #[br(big)] u32, // ← will be big-endian
//!     u32, // ← will be little-endian
//! );
//! ```
//!
//! ```
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead, Debug, PartialEq)]
//! #[br(big)]
//! struct MyType {
//!     val: u8,
//!     #[br(is_little = (val == 3))]
//!     other_val: u16 // ← little-endian if `val == 3`, otherwise big-endian
//! }
//!
//! # assert_eq!(MyType::read(&mut Cursor::new(b"\x03\x01\x00")).unwrap(), MyType { val: 3, other_val: 1 });
//! ```
//!
//! # Magic
//!
//! The `magic` directive matches [magic numbers](https://en.wikipedia.org/wiki/Magic_number_(programming))
//! in data:
//!
//! ```text
//! #[br(magic = $magic:literal)] or #[br(magic($magic:literal))]
//! ```
//!
//! The magic number can be a byte literal, byte string, char, float, or
//! integer. When a magic number is matched, parsing begins with the first byte
//! after the magic number in the data. When a magic number is not matched, an
//! error is returned.
//!
//! ## Examples
//!
//! ```
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
//! ## Errors
//!
//! If the specified magic number does not match the data, a
//! [`BadMagic`](crate::Error::BadMagic) error is returned and the reader’s
//! position is reset to where it was before parsing started.
//!
//! # Assert
//!
//! The `assert` directive validates objects and fields after they are read,
//! returning an error if the assertion condition evaluates to `false`:
//!
//! ```text
//! #[br(assert($cond:expr $(,)?))]
//! #[br(assert($cond:expr, $msg:literal $(,)?)]
//! #[br(assert($cond:expr, $fmt:literal, $($arg:expr),* $(,)?))]
//! #[br(assert($cond:expr, $err:expr $(,)?)]
//! ```
//!
//! Multiple assertion directives can be used; they will be combined and
//! executed in order.
//!
//! Assertions added to the top of an enum will be checked against every variant
//! in the enum.
//!
//! Any earlier field or [import](#arguments) can be referenced by expressions
//! in the directive.
//!
//! ## Examples
//!
//! ### Formatted error
//!
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(Debug, PartialEq)]
//! struct NotSmallerError(u32, u32);
//!
//! #[derive(BinRead, Debug)]
//! #[br(assert(some_val > some_smaller_val, "oops! {} <= {}", some_val, some_smaller_val))]
//! struct Test {
//!     some_val: u32,
//!     some_smaller_val: u32
//! }
//!
//! let error = Cursor::new(b"\0\0\0\x01\0\0\0\xFF").read_be::<Test>();
//! assert!(error.is_err());
//! let error = error.unwrap_err();
//! let expected = "oops! 1 <= 255".to_string();
//! assert!(matches!(error, binread::Error::AssertFail { message: expected, .. }));
//! ```
//!
//! ### Custom error
//!
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
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
//! ## Errors
//!
//! If the assertion fails and there is no second argument, or a string literal
//! is given as the second argument, an [`AssertFail`](crate::Error::AssertFail)
//! error is returned.
//!
//! If the assertion fails and an expression is given as the second argument,
//! a [`Custom`](crate::Error::Custom) error containing the result of the
//! expression is returned.
//!
//! Arguments other than the condition are not evaluated unless the assertion
//! fails, so it is safe for them to contain expensive operations without
//! impacting performance.
//!
//! In all cases, the reader’s position is reset to where it was before parsing
//! started.
//!
//! # Pre-assert
//!
//! `pre_assert` works like [`assert`](#assert), but checks the condition before
//! data is read instead of after. This is most useful when validating arguments
//! or choosing an enum variant to parse.
//!
//! ```text
//! #[br(pre_assert($cond:expr $(,)?))]
//! #[br(pre_assert($cond:expr, $msg:literal $(,)?)]
//! #[br(pre_assert($cond:expr, $fmt:literal, $($arg:expr),* $(,)?))]
//! #[br(pre_assert($cond:expr, $err:expr $(,)?)]
//! ```
//!
//! ## Examples
//!
//! ```
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead, Debug, PartialEq)]
//! #[br(import(ty: u8))]
//! enum Command {
//!     #[br(pre_assert(ty == 0))] Variant0(u16, u16),
//!     #[br(pre_assert(ty == 1))] Variant1(u32)
//! }
//!
//! #[derive(BinRead, Debug, PartialEq)]
//! struct Message {
//!     ty: u8,
//!     len: u8,
//!     #[br(args(ty))]
//!     data: Command
//! }
//!
//! let msg = Cursor::new(b"\x01\x04\0\0\0\xFF").read_be::<Message>();
//! assert!(msg.is_ok());
//! let msg = msg.unwrap();
//! assert_eq!(msg, Message { ty: 1, len: 4, data: Command::Variant1(0xFF) });
//! ```
//!
//! # Arguments
//!
//! The `import` and `args` directives define the type of
//! [`BinRead::Args`](crate::BinRead::Args) and the values passed in the `args`
//! argument of a [`BinRead::read_options`](crate::BinRead::read_options) call,
//! respectively:
//!
//! ```text
//! #[br(import($($ident:ident : $ty:ty),* $(,)?))]
//! #[br(args($($ident:ident),* $(,)?))]
//! ```
//!
//! Any earlier field or [import](#arguments) can be referenced in `args`.
//!
//! ## Examples
//!
//! ```
//! # use binread::prelude::*;
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
//! # Default
//!
//! The `default` directive, and its alias `ignore`, sets the value of the field
//! to its [`Default`](core::default::Default) instead of reading data from the
//! reader:
//!
//! ```text
//! #[br(default)] or #[br(ignore)]
//! ```
//!
//! ## Examples
//!
//! ```rust
//! # use binread::{BinRead, io::Cursor};
//! #[derive(BinRead, Debug, PartialEq)]
//! struct Test {
//!     #[br(default)]
//!     path: Option<std::path::PathBuf>,
//! }
//!
//! assert_eq!(
//!     Test::read(&mut Cursor::new(b"")).unwrap(),
//!     Test { path: None }
//! );
//! ```
//!
//! # Temp
//!
//! **This directive can only be used with [`derive_binread`]. It will not work
//! with `#[derive(BinRead)]`.**
//!
//! The `temp` directive causes a field to be treated as a temporary variable
//! instead of an actual field. The field will be removed from the struct
//! definition generated by [`derive_binread`]:
//!
//! ```text
//! #[br(temp)]
//! ```
//!
//! This allows data to be read which is necessary for parsing an object but
//! which doesn’t need to be stored in the final object. To skip data, entirely
//! use an [alignment directive](#padding-and-alignment) instead.
//!
//! ## Examples
//!
//! ```rust
//! # use binread::{BinRead, io::Cursor, derive_binread};
//! #[derive_binread]
//! #[derive(Debug, PartialEq)]
//! struct Test {
//!     // Since `Vec` stores its own length, this field is redundant
//!     #[br(temp, big)]
//!     len: u32,
//!
//!     #[br(count = len)]
//!     data: Vec<u8>
//! }
//!
//! assert_eq!(
//!     Test::read(&mut Cursor::new(b"\0\0\0\x05ABCDE")).unwrap(),
//!     Test { data: Vec::from(&b"ABCDE"[..]) }
//! );
//! ```
//!
//! # Postprocessing
//!
//! The `deref_now` directive, and its alias `postprocess_now`, cause a
//! field’s [`after_parse`](crate::BinRead::after_parse) function to be called
//! immediately after the field is parsed, instead of deferring the call until
//! the entire parent object has been parsed:
//!
//! ```text
//! #[br(deref_now)] or #[br(postprocess_now)]
//! ```
//!
//! The [`BinRead::after_parse`](crate::BinRead::after_parse) function is
//! normally used to perform additional work after the whole parent object has
//! been parsed. For example, the [`FilePtr`](crate::FilePtr) type reads an
//! object offset during parsing with
//! [`read_options`](crate::BinRead::read_options), then actually seeks to and
//! parses the pointed-to object in `after_parse`. This improves read
//! performance by reading the whole parent object sequentially before seeking
//! to read the pointed-to object.
//!
//! However, if another field in the parent object needs to access data from the
//! pointed-to object, `after_parse` needs to be called earlier. Adding
//! `deref_now` (or its alias, `postprocess_now`) to the earlier field causes
//! this to happen.
//!
//! ## Examples
//!
//! ```
//! # use binread::{prelude::*, FilePtr32, NullString, io::Cursor};
//! #[derive(BinRead, Debug)]
//! #[br(big, magic = b"TEST")]
//! struct TestFile {
//!     #[br(deref_now)]
//!     ptr: FilePtr32<NullString>,
//!
//!     value: i32,
//!
//!     // Notice how `ptr` can be used as it has already been postprocessed
//!     #[br(calc = ptr.len())]
//!     ptr_len: usize,
//! }
//!
//! # let test_contents = b"\x54\x45\x53\x54\x00\x00\x00\x10\xFF\xFF\xFF\xFF\x00\x00\x00\x00\x54\x65\x73\x74\x20\x73\x74\x72\x69\x6E\x67\x00\x00\x00\x00\x69";
//! # let test = Cursor::new(test_contents).read_be::<TestFile>().unwrap();
//! # assert_eq!(test.ptr_len, 11);
//! # assert_eq!(test.value, -1);
//! # assert_eq!(test.ptr.to_string(), "Test string");
//! ```
//!
//! # Restore position
//!
//! The `restore_position` directive restores the position of the reader after
//! a field is read:
//!
//! ```text
//! #[br(restore_position)]
//! ```
//!
//! To seek to an arbitrary position, use [`seek_before`](#padding-and-alignment)
//! instead.
//!
//! ## Examples
//!
//! ```
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead, Debug, PartialEq)]
//! struct MyType {
//!     #[br(restore_position)]
//!     test: u32,
//!     test_bytes: [u8; 4]
//! }
//!
//! # assert_eq!(
//! #   Cursor::new(b"\0\0\0\x01").read_be::<MyType>().unwrap(),
//! #   MyType { test: 1, test_bytes: [0,0,0,1]}
//! # );
//! ```
//!
//! ## Errors
//!
//! If querying or restoring the reader position fails, an
//! [`Io`](crate::Error::Io) error is returned and the reader’s
//! position is reset to where it was before parsing started.
//!
//! # Try
//!
//! The `try` directive allows parsing of an [`Option`] field to fail instead
//! of returning an error:
//!
//! ```text
//! #[br(try)]
//! ```
//!
//! If the field cannot be parsed, the position of the reader will be restored
//! and the value of the field will be set to [`None`].
//!
//! ## Examples
//!
//! ```
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(try)]
//!     maybe_u32: Option<u32>
//! }
//!
//! assert_eq!(Cursor::new(b"").read_be::<MyType>().unwrap().maybe_u32, None);
//! ```
//!
//! # Map
//!
//! The `map` and `try_map` directives allow data to be read using one type and
//! stored as another:
//!
//! ```text
//! #[br(map = $map_fn:expr)] or #[map($map_fn:expr))]
//! #[br(try_map = $map_fn:expr)] or #[try_map($map_fn:expr))]
//! ```
//!
//! When using `map` on a field, the map function must explicitly declare the
//! type of the data to be read in its first parameter and return a value which
//! matches the type of the field. The map function can be a plain function,
//! closure, or call expression which returns a plain function or closure.
//!
//! When using `try_map` on a field, the same rules apply, except that the
//! function must return a [`Result`] instead.
//!
//! When using `map` or `try_map` on a struct or enum, the map function must
//! return `Self` or `Result<Self, E>`.
//!
//! Any earlier field or [import](#arguments) can be referenced by the
//! expression in the directive.
//!
//! ## Examples
//!
//! ### Using `map` on a field
//!
//! ```
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(map = |x: u8| x.to_string())]
//!     int_str: String
//! }
//!
//! # assert_eq!(Cursor::new(b"\0").read_be::<MyType>().unwrap().int_str, "0");
//! ```
//!
//! ### Using `try_map` on a field
//!
//! ```
//! # use binread::{prelude::*, io::Cursor};
//! # use std::convert::TryInto;
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(try_map = |x: i8| x.try_into())]
//!     value: u8
//! }
//!
//! # assert_eq!(Cursor::new(b"\0").read_be::<MyType>().unwrap().value, 0);
//! # assert!(Cursor::new(b"\xff").read_be::<MyType>().is_err());
//! ```
//!
//! ### Using `map` on a struct to create a bit field
//!
//! The [`modular-bitfield`](https://docs.rs/modular-bitfield) crate can be used
//! along with `map` to create a struct out of raw bits.
//!
//! ```
//! # use binread::{prelude::*, io::Cursor};
//! use modular_bitfield::prelude::*;
//!
//! // This reads a single byte from the reader
//! #[bitfield]
//! #[derive(BinRead)]
//! #[br(map = Self::from_bytes)]
//! pub struct PackedData {
//!     status: B4,
//!     is_fast: bool,
//!     is_static: bool,
//!     is_alive: bool,
//!     is_good: bool,
//! }
//!
//! // example byte: 0x53
//! // [good] [alive] [static] [fast] [status]
//! //      0       1        0      1     0011
//! //  false    true    false   true        3
//!
//! # let data = Cursor::new(b"\x53").read_le::<PackedData>().unwrap();
//! # assert_eq!(data.is_good(), false);
//! # assert_eq!(data.is_alive(), true);
//! # assert_eq!(data.is_static(), false);
//! # assert_eq!(data.is_fast(), true);
//! # assert_eq!(data.status(), 3);
//! ```
//!
//! ## Errors
//!
//! If the `try_map` function returns a [`binread::io::Error`](crate::io::Error)
//! or [`std::io::Error`], an [`Io`](crate::Error::Io) error is returned. For
//! any other error type, a [`Custom`](crate::Error::Custom) error is returned.
//!
//! In all cases, the reader’s position is reset to where it was before parsing
//! started.
//!
//! # Custom parsers
//!
//! The `parse_with` directive specifies a custom parsing function which can be
//! used to override the default [`BinRead`](crate::BinRead) implementation for
//! a type, or to parse types which have no `BinRead` implementation at all:
//!
//! ```text
//! #[br(parse_with = $parse_fn:expr)] or #[br(parse_with($parse_fn:expr))]
//! ```
//!
//! Any earlier field or [import](#arguments) can be referenced by the
//! expression in the directive (for example, to construct a parser function at
//! runtime by calling a function generator).
//!
//! ## Examples
//!
//! ### Using a custom parser to generate a [`HashMap`](std::collections::HashMap)
//!
//! ```
//! # use binread::{prelude::*, io::*, ReadOptions};
//! # use std::collections::HashMap;
//! fn custom_parser<R: Read + Seek>(reader: &mut R, ro: &ReadOptions, _: ())
//!     -> BinResult<HashMap<u16, u16>>
//! {
//!     let mut map = HashMap::new();
//!     map.insert(
//!         reader.read_be().unwrap(),
//!         reader.read_be().unwrap()
//!     );
//!     Ok(map)
//! }
//!
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(parse_with = custom_parser)]
//!     offsets: HashMap<u16, u16>
//! }
//!
//! # assert_eq!(Cursor::new(b"\0\0\0\x01").read_be::<MyType>().unwrap().offsets.get(&0), Some(&1));
//! ```
//!
//! ### Using `FilePtr::parse` to read a `NullString` without storing a `FilePtr`
//!
//! ```
//! # use binread::{prelude::*, io::Cursor, FilePtr32, NullString};
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(parse_with = FilePtr32::parse)]
//!     some_string: NullString,
//! }
//!
//! # let val: MyType = Cursor::new(b"\0\0\0\x04Test\0").read_be().unwrap();
//! # assert_eq!(val.some_string.to_string(), "Test");
//! ```
//!
//! # Calculations
//!
//! The `calc` directive computes the value of a field instead of reading data
//! from the reader:
//!
//! ```text
//! #[br(calc = $value:expr)] or #[br(calc($value:expr))]
//! ```
//!
//! Any earlier field or [import](#arguments) can be referenced by the
//! expression in the directive.
//!
//! ## Examples
//!
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct MyType {
//!     var: u32,
//!     #[br(calc = 3 + var)]
//!     var_plus_3: u32,
//! }
//!
//! # assert_eq!(Cursor::new(b"\0\0\0\x01").read_be::<MyType>().unwrap().var_plus_3, 4);
//! ```
//!
//! # Count
//!
//! The `count` directive sets the number of values to read into a repeating
//! collection type like a [`Vec`]:
//!
//! ```text
//! #[br(count = $count:expr) or #[br(count($count:expr))]
//! ```
//!
//! When manually implementing
//! [`BinRead::read_options`](crate::BinRead::read_options) or a
//! [custom parser function](#custom-parsers), the `count` value is accessible
//! from [`ReadOptions::count`](crate::ReadOptions::count).
//!
//! Any earlier field or [import](#arguments) can be referenced by the
//! expression in the directive.
//!
//! ## Examples
//!
//! ### Using `count` with [`Vec`]
//!
//! ```
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct MyType {
//!     size: u32,
//!     #[br(count = size)]
//!     data: Vec<u8>,
//! }
//!
//! # assert_eq!(
//! #    Cursor::new(b"\0\0\0\x04\x01\x02\x03\x04").read_be::<MyType>().unwrap().data,
//! #    &[1u8, 2, 3, 4]
//! # );
//! ```
//!
//! ### Using `count` with [`FilePtr`](crate::FilePtr) and `Vec`
//!
//! ```
//! # use binread::{prelude::*, io::Cursor, FilePtr};
//! #[derive(BinRead)]
//! struct MyType {
//!     size: u32,
//!     #[br(count = size)]
//!     data: FilePtr<u32, Vec<u8>>,
//! }
//!
//! # assert_eq!(
//! #    *(Cursor::new(b"\0\0\0\x04\0\0\0\x09\0\x01\x02\x03\x04").read_be::<MyType>().unwrap().data),
//! #    &[1u8, 2, 3, 4]
//! # );
//! ```
//!
//! # Offset
//!
//! The `offset` and `offset_after` directives specify an additional relative
//! offset to a value accessed by a `BinRead` implementation which reads data
//! from an offset, like [`FilePtr`](crate::FilePtr):
//!
//! ```text
//! #[br(offset = $offset:expr)] or #[br(offset($offset:expr))]
//! #[br(offset_after = $offset:expr)] or #[br(offset_after($offset:expr))]
//! ```
//!
//! When manually implementing
//! [`BinRead::read_options`](crate::BinRead::read_options) or a
//! [custom parser function](#custom-parsers), the offset is accessible
//! from [`ReadOptions::offset`](crate::ReadOptions::offset).
//!
//! For `offset`, any earlier field or [import](#arguments) can be referenced by
//! the expression in the directive.
//!
//! For `offset_after`, *all* fields and imports can be referenced by the
//! expression in the directive, but [`deref_now`](#postprocessing) cannot be
//! used.
//!
//! ## Examples
//!
//! ```rust
//! # use binread::{prelude::*, io::Cursor, FilePtr};
//! #[derive(BinRead, Debug, PartialEq)]
//! struct OffsetTest {
//!     #[br(little, offset = 4)]
//!     test: FilePtr<u8, u16>
//! }
//!
//! # assert_eq!(
//! #   *OffsetTest::read(&mut Cursor::new(b"\0\xFF\xFF\xFF\x02\0")).unwrap().test,
//! #   2u16
//! # );
//! ```
//!
//! ## Errors
//!
//! If seeking to or reading from the offset fails, an [`Io`](crate::Error::Io)
//! error is returned and the reader’s position is reset to where it was before
//! parsing started.
//!
//! # Conditional values
//!
//! The `if` directive allows conditional parsing of a field, reading from data
//! if the condition is true and using a computed value if the condition is
//! false:
//!
//! ```text
//! #[br(if = $cond:expr)] or #[br(if($cond:expr))]
//! #[br(if = $cond:expr, $alternate:expr)] or #[br(if($cond:expr, $alternate:expr))]
//! ```
//!
//! If an alternate is provided, that value will be used when the condition is
//! false; otherwise, the [`default`](core::default::Default) value for the type
//! will be used.
//!
//! The alternate expression is not evaluated unless the condition is false, so
//! it is safe for it to contain expensive operations without impacting
//! performance.
//!
//! Any earlier field or [import](#arguments) can be referenced by the
//! expression in the directive.
//!
//! ## Examples
//!
//! ### Using an [`Option`] field with no alternate
//!
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct MyType {
//!     var: u32,
//!
//!     #[br(if(var == 1))]
//!     original_byte: Option<u8>,
//!
//!     #[br(if(var != 1))]
//!     other_byte: Option<u8>,
//! }
//!
//! # assert_eq!(Cursor::new(b"\0\0\0\x01\x03").read_be::<MyType>().unwrap().original_byte, Some(3));
//! # assert_eq!(Cursor::new(b"\0\0\0\x01\x03").read_be::<MyType>().unwrap().other_byte, None);
//! ```
//!
//! ### Using a scalar field with an explicit alternate
//!
//! ```rust
//! # use binread::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct MyType {
//!     var: u32,
//!
//!     #[br(if(var == 1, 0))]
//!     original_byte: u8,
//!
//!     #[br(if(var != 1, 42))]
//!     other_byte: u8,
//! }
//!
//! # assert_eq!(Cursor::new(b"\0\0\0\x01\x03").read_be::<MyType>().unwrap().original_byte, 3);
//! # assert_eq!(Cursor::new(b"\0\0\0\x01\x03").read_be::<MyType>().unwrap().other_byte, 42);
//! ```
//!
//! # Padding and alignment
//!
//! BinRead offers different directives for common forms of
//! [data structure alignment](https://en.wikipedia.org/wiki/Data_structure_alignment#Data_structure_padding).
//!
//! The `pad_before` and `pad_after` directives skip a specific number of bytes
//! either before or after reading a field, respectively:
//!
//! ```text
//! #[br(pad_after = $skip_bytes:expr)] or #[br(pad_after($skip_bytes:expr))]
//! #[br(pad_before = $skip_bytes:expr)] or #[br(pad_before($skip_bytes:expr))]
//! ```
//!
//! The `align_before` and `align_after` directives align the next read to the
//! given byte alignment either before or after reading a field, respectively:
//!
//! ```text
//! #[br(align_after = $align_to:expr)] or #[br(align_after($align_to:expr))]
//! #[br(align_before = $align_to:expr)] or #[br(align_before($align_to:expr))]
//! ```
//!
//! The `seek_before` directive accepts a [`SeekFrom`](crate::io::SeekFrom)
//! object and seeks the reader to an arbitrary position before reading a field:
//!
//! ```text
//! #[br(seek_before = $seek_from:expr)] or #[br(seek_before($seek_from:expr))]
//! ```
//!
//! The position of the reader will not be restored after the seek; use the
//! [`restore_position`](#restore-position) directive for this.
//!
//! The `pad_size_to` directive will ensure that the reader has advanced at
//! least the number of bytes given after the field has been read:
//!
//! ```text
//! #[br(pad_size_to = $size:expr)] or #[br(pad_size_to($size:expr))]
//! ```
//!
//! For example, if a format uses a null-terminated string, but always reserves
//! at least 256 bytes for that string, [`NullString`](crate::NullString) will
//! read the string and `pad_size_to(256)` will ensure the reader skips whatever
//! padding, if any, remains. If the string is longer than 256 bytes, no padding
//! will be skipped.
//!
//! Any earlier field or [import](#arguments) can be referenced by the
//! expressions in any of these directives.
//!
//! ## Examples
//!
//! ```rust
//! # use binread::{BinRead, NullString, io::SeekFrom};
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(align_before = 4, pad_after = 1, align_after = 4)]
//!     str: NullString,
//!
//!     #[br(pad_size_to = 0x10)]
//!     test: u64,
//!
//!     #[br(seek_before = SeekFrom::End(-4))]
//!     end: u32,
//! }
//! ```
//!
//! ## Errors
//!
//! If seeking fails, an [`Io`](crate::Error::Io) error is returned and the
//! reader’s position is reset to where it was before parsing started.
//!
//! # Repr
//!
//! The `repr` directive is used on a unit-like (C-style) enum to specify the
//! underlying type to use when reading the field and matching variants:
//!
//! ```text
//! #[br(repr = $ty:ty)] or #[br(repr($ty:ty))]
//! ```
//!
//! ## Examples
//!
//! ```
//! # use binread::BinRead;
//! #[derive(BinRead)]
//! #[br(big, repr = i16)]
//! enum FileKind {
//!     Unknown = -1,
//!     Text,
//!     Archive,
//!     Document,
//!     Picture,
//! }
//! ```
//!
//! ## Errors
//!
//! If a read fails, an [`Io`](crate::Error::Io) error is returned. If no
//! variant matches, a [`NoVariantMatch`](crate::Error::NoVariantMatch) error
//! is returned.
//!
//! In all cases, the reader’s position is reset to where it was before parsing
//! started.

#![allow(unused_imports)]

use crate::derive_binread;
