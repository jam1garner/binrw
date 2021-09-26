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
//! | [`write_with`](#custom-parsers) | field | Specifies a custom function for writing a field.
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
//! todo
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
//! #[br(calc = $value:expr)] or #[br(calc($value:expr))]
//! ```
//!
//! Any field (earlier or later) or [import](#arguments) can be referenced by the
//! expression in the directive.
//!
//! **Note:** within `BinWrite` calc removes the field from the struct, similarly to
//! `#[br(temp)]`. The field also needs to be marked `#[br(temp)]` in order to ensure
//! the parser does not try and store a value in the non-existent field.
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
//! todo
