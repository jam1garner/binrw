//! Documentation of directives of the `#[bw]` attribute
//!
//! (This is currently in a WIP state and likely quite broken)
//!
//! # List of directives
//!
//! | Directive | Supports | Description
//! |-----------|----------|------------
//! | [`align_after`](#padding-and-alignment) | field | Aligns the writer to the Nth byte after writing data.
//! | [`align_before`](#padding-and-alignment) | field | Aligns the writer to the Nth byte before writing data.
//! | [`args`](#arguments) | struct field, data variant | Passes arguments to another `BinWrite` type.
//! | [`args_raw`](#arguments) | struct field, data variant | Like `args`, but specifies a type containing the arguments.
//! | [`assert`](#assert) | struct, field, non-unit enum, data variant | Asserts that a condition is true before writing.
//! | [`big`](#byte-order) | all except unit variant | Sets the byte order to big-endian.
//! | [`calc`](#calculations) | field | Computes the value of a field instead of pulling the value from a struct. Removes the field from the actual type definition.
//! | [`ignore`](#ignore) | field | Skip writing the field.
//! | [`import`](#arguments) | struct, non-unit enum, unit-like enum | Defines extra arguments for a struct or enum.
//! | [`import_tuple`](#arguments) | struct, non-unit enum, unit-like enum | Like `import`, but receives the arguments as a tuple.
//! | [`is_big`](#byte-order) | field | Conditionally sets the byte order to big-endian.
//! | [`is_little`](#byte-order) | field | Conditionally set the byte order to little-endian.
//! | [`little`](#byte-order) | all except unit variant | Sets the byte order to little-endian.
//! | [`magic`](#magic) | all | Writes a magic constant.
//! | [`map`](#map) | all except unit variant | Maps a value before writing. When used in the top-level position, the map function must take `Self`.
//! | [`pad_after`](#padding-and-alignment) | field | Writes N bytes of padding after writing the field.
//! | [`pad_before`](#padding-and-alignment) | field | Writes N bytes of padding before writing the field.
//! | [`pad_size_to`](#padding-and-alignment) | field | Ensures the writer is at least N bytes after the starting position for this field.
//! | [`write_with`](#custom-writers) | field | Specifies a custom function for writing a field.
//! | [`repr`](#repr) | unit-like enum | Specifies the underlying type for a unit-like (C-style) enum.
//! | [`restore_position`](#restore-position) | field | Restores the writer’s position after writing a field.
//! | [`seek_before`](#padding-and-alignment) | field | Moves the writer to a specific position before writing data.
//! | [`try_map`](#map) | all except unit variant | Like `map`, but returns a [`BinResult`](crate::BinResult).
//!
//!
//! # Padding and alignment
//!
//! BinWrite offers different directives for common forms of
//! [data structure alignment](https://en.wikipedia.org/wiki/Data_structure_alignment#Data_structure_padding).
//!
//! The `pad_before` and `pad_after` directives skip a specific number of bytes
//! either before or after writing a field, respectively:
//!
//! ```text
//! #[bw(pad_after = $skip_bytes:expr)] or #[bw(pad_after($skip_bytes:expr))]
//! #[bw(pad_before = $skip_bytes:expr)] or #[bw(pad_before($skip_bytes:expr))]
//! #[brw(pad_after = $skip_bytes:expr)] or #[brw(pad_after($skip_bytes:expr))]
//! #[brw(pad_before = $skip_bytes:expr)] or #[brw(pad_before($skip_bytes:expr))]
//! ```
//!
//! This is effectively equivelant to:
//!
//! ```rust
//! # let mut pos = 0;
//! # let padding = 0x4;
//! pos += padding;
//! ```
//!
//! ---
//!
//! The `align_before` and `align_after` directives align the next written byte to the
//! given byte alignment either before or after writing a field, respectively:
//!
//! ```text
//! #[bw(align_after = $align_to:expr)] or #[bw(align_after($align_to:expr))]
//! #[bw(align_before = $align_to:expr)] or #[bw(align_before($align_to:expr))]
//! #[brw(align_after = $align_to:expr)] or #[brw(align_after($align_to:expr))]
//! #[brw(align_before = $align_to:expr)] or #[brw(align_before($align_to:expr))]
//! ```
//!
//!  This is effectively equivelant to:
//!
//!  ```rust
//!  # let mut pos = 0;
//!  # let align = 0x10;
//!  if pos % align != 0 {
//!     pos += align - (pos % align);
//!  }
//!  ```
//!
//! # Arguments
//!
//! The `import` and `args` directives define the type of
//! [`BinWrite::Args`](crate::BinWrite::Args) and the values passed in the `args`
//! argument of a [`BinWrite::write_options`](crate::BinWrite::write_options) call.
//!
//! Any field or [import](#arguments) can be referenced in `#[bw(args)]`.
//!
//! There are 3 types of arguments:
//!
//! * Tuple-styled Arguments (Alternatively "Ordered Arguments") - arguments passed as a tuple
//! * Named arguments - arguments passed as a builder that ensures all required arguments are
//! passed (can be manually constructed using [`binrw::args`])
//! * Raw arguments - the arguments are passed as a type of your choice
//!
//! ## Examples
//!
//! ### Tuple-styled arguments
//!
//! Tuple-styled arguments are passed via `args()` and recieved via `import()`.
//!
//! ```
//! # use binrw::prelude::*;
//! #[derive(BinWrite)]
//! #[bw(import(val1: u32, val2: &'static str))]
//! struct ImportTest {
//!     // ...
//! }
//!
//! #[derive(BinWrite)]
//! struct ArgsTets {
//!     val: u32,
//!     #[bw(args(val + 3, "test"))]
//!     test: ImportTest
//! }
//! ```
//!
//! ### Named arguments
//!
//! Named arguments are passed via `args {}` and recieved via `import {}`. (Note the curly braces)
//!
//! ```
//! # use binrw::prelude::*;
//! #[derive(BinWrite)]
//! #[bw(import { count: u32, other: u16 = 0 })]
//! struct ImportTest {
//!     // ...
//! }
//!
//! #[derive(BinWrite)]
//! struct ArgsTets {
//!     count: u32,
//!
//!     #[bw(args { count: *count, other: 5 })]
//!     test: ImportTest,
//!
//!     #[bw(args { count: 3 })]
//!     test2: ImportTest,
//! }
//! ```
//!
//! The syntax is designed to mimic Rust's struct literal syntax. Another feature of named imports
//! is allowing to specify a default value in the form of `name: type = value`, which makes
//! passing the argument optional.
//!
//! ### Raw arguments
//!
//! Raw arguments can be used to have a higher degree of control over the type of the arguments
//! variable being passed into the writer.
//!
//! ```
//! # use binrw::prelude::*;
//!
//! #[derive(BinWrite)]
//! #[bw(import_raw(args: (u32, u16)))]
//! struct ImportTest {
//!     // ...
//! }
//!
//! #[derive(BinWrite)]
//! struct ArgsTets {
//!     count: u32,
//!
//!     #[bw(args(1, 2))]
//!     test: ImportTest,
//!
//!     // identical to the above
//!     #[bw(args_raw = (1, 2))]
//!     test2: ImportTest,
//! }
//! ```
//!
//! One common use of `import_raw` and `args_raw` is for easily forwarding arguments through to an
//! inner field of the structure.
//!
//! ## Technical notes
//!
//! The format for the import and args directives are as follows:
//!
//! ```text
//! // tuple-styled args
//! #[bw(import($($ident:ident : $ty:ty),* $(,)?))]
//! #[bw(args($($value:expr),* $(,)?))]
//!
//! // named args
//! #[bw(import{ $($ident:ident : $ty:ty $(= $default:expr)? ),* $(,)? })]
//! #[bw(args { $($name:ident $(: $value:expr)? ),* $(,)? } )]
//!
//! // raw args
//! #[bw(import_raw( $binding:ident : $ty:ty ))]
//! #[bw(args_raw($value:expr))]
//! #[bw(args_raw = $value:expr)] // same as above, alternative syntax
//! ```
//!
//! A notable limitation of the arguments system is not allowing non-static lifetimes. This is due
//! to the fact arguments desugar into approximately the following:
//!
//! ```rust,ignore
//! impl BinWrite for MyType {
//!     type Args = $ty;
//!
//!     fn write_options(..., args: Self::Args) -> Result<(), binrw::Error> {
//!         // ...
//!     }
//! }
//! ```
//!
//! Which, due to the fact the associated type `Args` cannot have a lifetime tied to the associated
//! function `write_options`, the type is inexpressible without [GATs](https://github.com/rust-lang/rfcs/pull/1598).
//!
//!
//! # Assert
//!
//! The `assert` directive validates objects and fields before they are written,
//! returning an error if the assertion condition evaluates to `false`:
//!
//! ```text
//! #[bw(assert($cond:expr $(,)?))]
//! #[bw(assert($cond:expr, $msg:literal $(,)?)]
//! #[bw(assert($cond:expr, $fmt:literal, $($arg:expr),* $(,)?))]
//! #[bw(assert($cond:expr, $err:expr $(,)?)]
//! ```
//!
//! Multiple assertion directives can be used; they will be combined and
//! executed in order.
//!
//! Assertions added to the top of an enum will be checked against every variant
//! in the enum.
//!
//! Any field or [import](#arguments) can be referenced by expressions
//! in the directive.
//!
//! ## Examples
//!
//! ### Formatted error
//!
//! ```rust
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(Debug, PartialEq)]
//! struct NotSmallerError(u32, u32);
//!
//! #[derive(BinWrite, Debug)]
//! #[bw(assert(some_val > some_smaller_val, "oops! {} <= {}", some_val, some_smaller_val))]
//! struct Test {
//!     some_val: u32,
//!     some_smaller_val: u32
//! }
//!
//! let mut writer = Cursor::new(Vec::new());
//! let err = writer.write_be(&Test{ some_val: 1, some_smaller_val: 3 }).unwrap_err();
//! assert!(matches!(err.root_cause(), binrw::Error::AssertFail { .. }));
//! ```
//!
//! ### Custom error
//!
//! ```rust
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(Debug, PartialEq)]
//! struct NotSmallerError(u32, u32);
//! impl core::fmt::Display for NotSmallerError {
//!     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//!         write!(f, "{} <= {}", self.0, self.1)
//!     }
//! }
//!
//! #[derive(BinWrite, Debug)]
//! #[bw(assert(some_val > some_smaller_val, NotSmallerError(*some_val, *some_smaller_val)))]
//! struct Test {
//!     some_val: u32,
//!     some_smaller_val: u32
//! }
//!
//! let mut writer = Cursor::new(Vec::new());
//! let err = writer.write_be(&Test { some_val: 1, some_smaller_val: 3 }).unwrap_err();
//! assert_eq!(err.custom_err(), Some(&NotSmallerError(1, 3)));
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
//! In all cases, the writer’s position is reset to where it was before parsing
//! started.
//!
//! # Byte order
//!
//! The `big` and `little` directives specify the [byte order](https://en.wikipedia.org/wiki/Endianness)
//! of data in a struct, enum, variant, or field:
//!
//! ```text
//! #[bw(big)]
//! #[bw(little)]
//! ```
//!
//! The `is_big` and `is_little` directives conditionally set the byte order of
//! a struct field:
//!
//! ```text
//! #[bw(is_little = $cond:expr)] or #[bw(is_little($cond:expr))]
//! #[bw(is_big = $cond:expr)] or #[bw(is_big($cond:expr))]
//! ```
//!
//! The `is_big` and `is_little` directives are primarily useful when byte order
//! is defined in the data itself. Any field or [import](#arguments) can
//! be referenced in the condition. Conditional byte order directives can only
//! be used on struct fields.
//!
//! The order of precedence (from highest to lowest) for determining byte order
//! within an object is:
//!
//! 1. A directive on a field
//! 2. A directive on an enum variant
//! 3. A directive on the struct or enum
//! 4. The [`endian`](crate::WriteOptions::endian) property of the
//!    [`WriteOptions`](crate::WriteOptions) object passed to
//!    [`BinWrite::write_options`](crate::BinWrite::write_options) by the caller
//! 5. The host machine’s native byte order
//!
//!
//! # Calculations
//!
//! The `calc` directive computes the value of a field instead of writing the value
//! from the type itself.
//!
//! ```text
//! #[bw(calc = $value:expr)] or #[bw(calc($value:expr))]
//! ```
//!
//! Any field (earlier or later) or [import](#arguments) can be referenced by the
//! expression in the directive.
//!
//! **Note:** within `BinWrite` calc removes the field from the struct, similarly to
//! `#[br(temp)]`. When both `BinRead` and `BinWrite` is implemented, the field also needs to be
//! marked `#[br(temp)]` in order to ensure the reader does not try and store a value in the
//! non-existent field.
//!
//! ## Examples
//!
//! A simple example showing how calc is necessary for writing an array prefixed
//! by a count:
//!
//! ```rust
//! # use binrw::{binwrite, prelude::*, io::Cursor};
//! #[binwrite]
//! struct MyType {
//!     #[bw(calc = items.len() as u32)]
//!     size: u32,
//!     items: Vec<u8>,
//! }
//!
//! let mut writer = Cursor::new(Vec::new());
//! writer.write_be(&MyType { items: vec![0, 1, 2] }).unwrap();
//! # assert_eq!(&writer.into_inner()[..], &[0, 0, 0, 3, 0, 1, 2]);
//! ```
//!
//! And another example showing how `#[br(temp)]` is needed when making this round-trip:
//!
//! ```rust
//! # use binrw::{binrw, prelude::*, io::Cursor};
//! #[binrw]
//! struct MyType {
//!     #[br(temp)]
//!     #[bw(calc = items.len() as u32)]
//!     size: u32,
//!
//!     #[br(count = size)]
//!     items: Vec<u8>,
//! }
//!
//! let list: MyType = Cursor::new(b"\0\0\0\x03\0\x01\x02").read_be().unwrap();
//! let mut writer = Cursor::new(Vec::new());
//! writer.write_be(&list).unwrap();
//! # assert_eq!(&writer.into_inner()[..], b"\0\0\0\x03\0\x01\x02");
//! ```
//!
//! # Default
//!
//! The `default` directive, and its alias `ignore`, sets the value of the field
//! to its [`Default`](core::default::Default) instead of dumping data from the
//! writer:
//!
//! ```text
//! #[bw(default)] or #[bw(ignore)]
//! #[brw(default)] or #[brw(ignore)]
//! ```
//!
//! ## Examples
//!
//! ```rust
//! # use binrw::{binrw, prelude::*, BinWrite, io::Cursor};
//! #[binrw]
//! #[bw(import { x: u32, _y: u8 })]
//! struct MyStruct {
//!     #[br(temp, ignore)]
//!     #[bw(calc = x)]
//!     x_copy: u32,
//! }
//! let mut x = binrw::io::Cursor::new(Vec::new());
//! MyStruct {}
//!     .write_options(&mut x, &Default::default(), binrw::args! { x: 3, _y: 2 })
//!     .unwrap();
//! ```
//!
//! The `magic` directive matches [magic numbers](https://en.wikipedia.org/wiki/Magic_number_(programming))
//! in data:
//!
//! ```text
//! #[bw(magic = $magic:literal)] or #[bw(magic($magic:literal))]
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
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinWrite, Debug)]
//! #[bw(magic = b"TEST")]
//! struct Test {
//!     val: u32
//! }
//!
//! #[derive(BinWrite, Debug)]
//! #[bw(magic = 1.2f32)]
//! struct Version(u16);
//!
//! #[derive(BinWrite)]
//! enum Command {
//!     #[bw(magic = 0u8)] Nop,
//!     #[bw(magic = 1u8)] Jump { loc: u32 },
//!     #[bw(magic = 2u8)] Begin { var_count: u16, local_count: u16 }
//! }
//! ```
//!
//! ## Errors
//!
//! If the specified magic number does not match the data, a
//! [`BadMagic`](crate::Error::BadMagic) error is returned and the writer’s
//! position is reset to where it was before parsing started.
//!
//! # Map
//!
//! The `map` and `try_map` directives allow data to be written using one type and
//! stored as another:
//!
//! ```text
//! #[bw(map = $map_fn:expr)] or #[map($map_fn:expr))]
//! #[bw(try_map = $map_fn:expr)] or #[try_map($map_fn:expr))]
//! ```
//!
//! When using `map` on a field, the map function must explicitly declare the
//! type of the data to be written in its first parameter and return a value which
//! matches the type of the field. The map function can be a plain function,
//! closure, or call expression which returns a plain function or closure.
//!
//! When using `try_map` on a field, the same rules apply, except that the
//! function must return a [`Result`] instead.
//!
//! When using `map` or `try_map` on a struct or enum, the map function must
//! return `Self` or `Result<Self, E>`.
//!
//! Any field or [import](#arguments) can be referenced by the
//! expression in the directive.
//!
//! ## Examples
//!
//! ### Using `map` on a field
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinWrite)]
//! struct MyType {
//!     #[bw(map = |x: &String| -> u8 { x.parse().unwrap() })]
//!     int_str: String
//! }
//!
//! let mut writer = Cursor::new(Vec::new());
//! writer.write_be(&MyType { int_str: "1".to_string() }).unwrap();
//! assert_eq!(&writer.into_inner()[..], b"\x01")
//! ```
//!
//! ### Using `try_map` on a field
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! # use core::convert::TryInto;
//! #[derive(BinWrite)]
//! struct MyType {
//!     #[bw(try_map = |&x| -> BinResult<i8> { x.try_into().map_err(|_| todo!()) })]
//!     value: u8
//! }
//!
//! let mut writer = Cursor::new(Vec::new());
//! writer.write_be(&MyType { value: 3 });
//! assert_eq!(&writer.into_inner()[..], b"\x03")
//! ```
//!
//! ### Using `map` on a struct to create a bit field
//!
//! The [`modular-bitfield`](https://docs.rs/modular-bitfield) crate can be used
//! along with `map` to create a struct out of raw bits.
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! use modular_bitfield::prelude::*;
//!
//! // The cursor dumps a single byte
//! #[bitfield]
//! #[derive(BinWrite, Clone, Copy)]
//! #[bw(map = |&x| Self::into_bytes(x))]
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
//! # let mut writer = Cursor::new(Vec::new());
//! # writer.write_le(&PackedData::new().with_status(3)).unwrap();
//! # assert_eq!(&writer.into_inner()[..], b"\x03");
//! ```
//!
//! ## Errors
//!
//! If the `try_map` function returns a [`binrw::io::Error`](crate::io::Error) or
//! a [`std::io::Error`](crate::io::Error), an [`Io`](crate::Error::Io) error is returned. For
//! any other error type, a [`Custom`](crate::Error::Custom) error is returned.
//!
//! In all cases, the writer’s position is reset to where it was before parsing
//! started.
//!
//! # Repr
//!
//! The `repr` directive is used on a unit-like (C-style) enum to specify the
//! underlying type to use when reading the field and matching variants:
//!
//! ```text
//! #[br(repr = $ty:ty)] or #[br(repr($ty:ty))]
//! #[brw(repr = $ty:ty)] or #[brw(repr($ty:ty))]
//! ```
//!
//! ## Examples
//!
//! ```
//! # use binrw::BinWrite;
//! #[derive(BinWrite)]
//! #[bw(big, repr = i16)]
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
//! In all cases, the writer’s position is reset to where it was before parsing
//! started.
//!
//!
//! # Restore Position
//!
//! The `restore_position` directive restores the position of the writer after
//! a field is writen:
//!
//! ```text
//! #[bw(restore_position)]
//! ```
//!
//! To seek to an arbitrary position, use [`seek_before`](#padding-and-alignment).
//!
//! ## Example
//!
//! ```
//! # use binrw::{binwrite, prelude::*, io::Cursor};
//!
//! #[binwrite]
//! struct MyType {
//!     #[bw(restore_position)]
//!     my_u24: u32,
//!     override_byte: u8,
//! }
//!
//! let mut writer = Cursor::new(Vec::new());
//! writer.write_be(&MyType { my_u24: 3, override_byte: 1 }).unwrap();
//! assert_eq!(&writer.into_inner()[..], b"\x01\x00\x00\x03");
//! ```
//! Here a u32 (my_u24) is written, then the first byte is overwritten by the byte "override_byte".
//! # Custom writers
//!
//! The `write_with` directive specifies a custom writing function that can be
//! used to override the default [`BinWrite`](crate::BinWrite) implementation for
//! a type, or to dump values which have no `BinWrite` implementation at all:
//!
//! ```text
//! #[bw(write_with = $write_fn:expr)] or #[bw(write_with($write_fn:expr))]
//! #[brw(write_with = $write_fn:expr)] or #[brw(write_with($write_fn:expr))]
//! ```
//!
//! Any field or [import](#arguments) can be referenced by the
//! expression in the directive (for example, to construct a writer function by
//! passing a value to a function that returns a closure).
//!
//! ## Examples
//!
//! ### Using a custom writer
//!
//! ```
//! # use binrw::{prelude::*, io::*, BinWrite, Endian, WriteOptions};
//! fn custom_writer<W: binrw::io::Write + binrw::io::Seek>(
//!     &amount: &u16,
//!     writer: &mut W,
//!     _opts: &WriteOptions,
//!     _: (),
//! ) -> binrw::BinResult<()> {
//!     for _ in 0..amount {
//!         writer.write_all(b"abcd")?;
//!     }
//!     Ok(())
//! }
//! #[derive(BinWrite)]
//! struct MyData {
//!     x: u8,
//!     #[bw(write_with = custom_writer)]
//!     y: u16,
//! }
//! fn dump_mydata() {
//!     let mut x = Cursor::new(Vec::new());
//!     MyData { x: 1, y: 2 }
//!         .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
//!         .unwrap();
//!     assert_eq!(&x.into_inner()[..], b"\x01abcdabcd");
//! }
//! ```
