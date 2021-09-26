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
//! # Padding and Alignment
//!
//! todo
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
//! todo
//!
//! # Byte Order
//!
//! todo
//!
//! # Caculations
//!
//! The `calc` directive computes the value of a field instead of reading the value
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
//! `#[bw(temp)]`. The field also needs to be marked `#[bw(temp)]` in order to ensure
//! the writer does not try and store a value in the non-existent field.
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
//! And another example showing how `#[bw(temp)]` is needed when making this round-trip:
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
//! # Ignore
//!
//! todo
//!
//! # Magic
//!
//! todo
//!
//! # Map
//!
//! todo
//!
//! # Repr
//!
//! todo
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
//!
//! Here a u32 (my_u24) is written, then the first byte is overwritten by the byte "override_byte".
//!
//! # Custom Writers
//!
//! todo
