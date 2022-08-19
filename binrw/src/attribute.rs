//! A documentation-only module for binrw directives.
//!
//! <style>
//!     .show-rw {
//!         background: var(--color-background, #fff);
//!         border: thin solid #ddd;
//!         box-shadow: -0.25rem 0.25rem 0 var(--color-background, #fff);
//!         display: flex;
//!         list-style: none;
//!         margin: 0 0 -2.25rem auto;
//!         gap: 0.5rem;
//!         padding: 0.25rem 1rem 0.75rem;
//!         position: sticky;
//!         top: 2rem;
//!         width: fit-content;
//!         z-index: 1;
//!     }
//!     .show-rw > legend {
//!         background: var(--color-background, #fff);
//!         padding: 0 0.15rem;
//!     }
//!     .show-rw [for]::before {
//!         background-clip: content-box;
//!         border: thin solid var(--color-standard, #000);
//!         border-radius: 0.5rem;
//!         content: '';
//!         display: inline-block;
//!         height: 0.5rem;
//!         padding: 1px;
//!         width: 0.5rem;
//!     }
//!     #show_write:checked ~ .br:not(.bw),
//!     #show_write:checked ~ * .br:not(.bw),
//!     #show_read:checked ~ .bw:not(.br),
//!     #show_read:checked ~ * .bw:not(.br) {
//!         display: none;
//!     }
//!     #show_read:checked ~ .show-rw [for=show_read]::before,
//!     #show_write:checked ~ .show-rw [for=show_write]::before {
//!         background-color: var(--color-standard, #000);
//!     }
//!     .br, .bw {
//!         display: contents;
//!     }
//! </style>
//! <input name="show_rw" id="show_read" type="radio" hidden checked>
//! <input name="show_rw" id="show_write" type="radio" hidden>
//! <fieldset class="show-rw">
//!   <legend>View for:</legend>
//!   <label for="show_read"><code>#[br]</code></label>
//!   <label for="show_write"><code>#[bw]</code></label>
//! </fieldset>
//!
//! # List of directives
//!
//! | r/w | Directive | Supports | Description
//! |-----|-----------|----------|------------
//! | rw  | [`align_after`](#padding-and-alignment) | field | Aligns the <span class="br">reader</span><span class="bw">writer</span> to the Nth byte after a field.
//! | rw  | [`align_before`](#padding-and-alignment) | field | Aligns the <span class="br">reader</span><span class="bw">writer</span> to the Nth byte before a field.
//! | rw  | [`args`](#arguments) | struct field, data variant | Passes arguments to another binrw object.
//! | rw  | [`args_raw`](#arguments) | struct field, data variant | Like `args`, but specifies a tuple containing the arguments.
//! | rw  | [`assert`](#assert) | struct, field, non-unit enum, data variant | Asserts that a condition is true. Can be used multiple times.
//! | rw  | [`big`](#byte-order) | all except unit variant | Sets the byte order to big-endian.
//! | rw  | [`calc`](#calculations) | field | Computes the value of a field instead of reading data.
//! | r   | [`count`](#count) | field | Sets the length of a vector.
//! | r   | [`default`](#ignore) | field | An alias for `ignore`.
//! | r   | [`deref_now`](#postprocessing) | field | An alias for `postprocess_now`.
//! | r   | [`err_context`](#backtrace) | field | Adds additional context to errors.
//! | r   | [`if`](#conditional-values) | field | Reads data only if a condition is true.
//! | rw  | [`ignore`](#ignore) | field | <span class="br">Uses the [`default`](core::default::Default) value for a field instead of reading data.</span><span class="bw">Skips writing the field.</span>
//! | rw  | [`import`](#arguments) | struct, non-unit enum, unit-like enum | Defines extra arguments for a struct or enum.
//! | rw  | [`import_tuple`](#arguments) | struct, non-unit enum, unit-like enum | Like `import`, but receives the arguments as a tuple.
//! | rw  | [`is_big`](#byte-order) | field | Conditionally sets the byte order to big-endian.
//! | rw  | [`is_little`](#byte-order) | field | Conditionally set the byte order to little-endian.
//! | rw  | [`little`](#byte-order) | all except unit variant | Sets the byte order to little-endian.
//! | rw  | [`magic`](#magic) | all | <span class="br">Matches</span><span class="bw">Writes</span> a magic number.
//! | rw  | [`map`](#map) | all except unit variant | Maps an object or value to a new value.
//! | r   | [`offset`](#offset) | field | Modifies the offset used by a [`FilePtr`](crate::FilePtr).
//! | rw  | [`pad_after`](#padding-and-alignment) | field | Skips N bytes after <span class="br">reading</span><span class="bw">writing</span> a field.
//! | rw  | [`pad_before`](#padding-and-alignment) | field | Skips N bytes before <span class="br">reading</span><span class="bw">writing</span> a field.
//! | rw  | [`pad_size_to`](#padding-and-alignment) | field | Ensures the <span class="br">reader</span><span class="bw">writer</span> is always advanced at least N bytes.
//! | r   | [`parse_with`](#custom-parserswriters) | field | Specifies a custom function for reading a field.
//! | r   | [`postprocess_now`](#postprocessing) | field | Calls [`after_parse`](crate::BinRead::after_parse) immediately after reading data instead of after all fields have been read.
//! | r   | [`pre_assert`](#pre-assert) | struct, non-unit enum, unit variant | Like `assert`, but checks the condition before parsing.
//! | rw  | [`repr`](#repr) | unit-like enum | Specifies the underlying type for a unit-like (C-style) enum.
//! | rw  | [`restore_position`](#restore-position) | field | Restores the <span class="br">reader’s</span><span class="bw">writer’s</span> position after <span class="br">reading</span><span class="bw">writing</span> a field.
//! | r   | [`return_all_errors`](#enum-errors) | non-unit enum | Returns a [`Vec`] containing the error which occurred on each variant of an enum on failure. This is the default.
//! | r   | [`return_unexpected_error`](#enum-errors) | non-unit enum | Returns a single generic error on failure.
//! | rw  | [`seek_before`](#padding-and-alignment) | field | Moves the <span class="br">reader</span><span class="bw">writer</span> to a specific position before <span class="br">reading</span><span class="bw">writing</span> data.
//! | r   | [`temp`](#temp) | field | Uses a field as a temporary variable. Only usable with the [`macro@binread`] attribute macro.
//! | r   | [`try`](#try) | field | Tries to parse and stores the [`default`](core::default::Default) value for the type if parsing fails instead of returning an error.
//! | rw  | [`try_map`](#map) | all except unit variant | Like `map`, but returns a [`BinResult`](crate::BinResult).
//! |  w  | [`write_with`](#custom-parserswriters) | field | Specifies a custom function for writing a field.
//!
//! # Arguments
//!
//! Arguments provide extra data necessary for
//! <span class="br">reading</span><span class="bw">writing</span> an
//! object.
//!
//! The `import` and `args` directives define the type of
//! <span class="br">[`BinRead::Args`](crate::BinRead::Args)</span>
//! <span class="bw">[`BinWrite::Args`](crate::BinWrite::Args)</span>
//! and the values passed in the `args`
//! argument of a
//! <span class="br">[`BinRead::read_options`](crate::BinRead::read_options)</span>
//! <span class="bw">[`BinWrite::write_options`](crate::BinWrite::write_options)</span>
//! call, respectively.
//!
//! Any earlier field or [import](#arguments) can be referenced in `args`.
//!
//! ## Ways to pass and receive arguments
//!
//! There are 3 ways arguments can be passed and received:
//!
//! * Tuple-style arguments (or “ordered arguments”): arguments passed as a tuple
//! * Named arguments: arguments passed as an object, using a builder that
//!   ensures all required arguments are given, or manually constructed using
//!   [`binrw::args`]
//! * Raw arguments: arguments passed as a type of your choice
//!
//! ### Tuple-style arguments
//!
//! Tuple-style arguments (or “ordered arguments”) are passed via `args()` and
//! received via `import()`:
//!
//! <div class="br">
//!
//! ```text
//! #[br(import($($ident:ident : $ty:ty),* $(,)?))]
//! #[br(args($($value:expr),* $(,)?))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(import($($ident:ident : $ty:ty),* $(,)?))]
//! #[bw(args($($value:expr),* $(,)?))]
//! ```
//! </div>
//!
//! This is the most common form of argument passing because it works mostly
//! like a normal function call:
//!
//! <div class="br">
//!
//! ```
//! # use binrw::prelude::*;
//! #[derive(BinRead)]
//! #[br(import(val1: u32, val2: &'static str))]
//! struct Child {
//!     // ...
//! }
//!
//! #[derive(BinRead)]
//! struct Parent {
//!     val: u32,
//!     #[br(args(val + 3, "test"))]
//!     test: Child
//! }
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::prelude::*;
//! #[derive(BinWrite)]
//! #[bw(import(val1: u32, val2: &'static str))]
//! struct Child {
//!     // ...
//! }
//!
//! #[derive(BinWrite)]
//! struct Parent {
//!     val: u32,
//!     #[bw(args(val + 3, "test"))]
//!     test: Child
//! }
//! ```
//! </div>
//!
//! ### Named arguments
//!
//! Named arguments are passed via `args {}` and received via `import {}`
//! (note the curly braces), similar to a struct literal:
//!
//! <div class="br">
//!
//! ```text
//! #[br(import { $($ident:ident : $ty:ty $(= $default:expr)?),* $(,)? })]
//! #[br(args { $($name:ident $(: $value:expr)?),* $(,)? } )]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(import { $($ident:ident : $ty:ty $(= $default:expr)?),* $(,)? })]
//! #[bw(args { $($name:ident $(: $value:expr)?),* $(,)? } )]
//! ```
//! </div>
//!
//! [Field init shorthand](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#using-the-field-init-shorthand)
//! and optional arguments are both supported.
//!
//! Named arguments are particularly useful for container objects like [`Vec`],
//! but they can be used by any <span class="br">parser</span><span class="bw">serialiser</span>
//! that would benefit from labelled, optional, or unordered arguments:
//!
//! <div class="br">
//!
//! ```
//! # use binrw::prelude::*;
//! #[derive(BinRead)]
//! #[br(import {
//!     count: u32,
//!     other: u16 = 0 // ← optional argument
//! })]
//! struct Child {
//!     // ...
//! }
//!
//! #[derive(BinRead)]
//! struct Parent {
//!     count: u32,
//!
//!     #[br(args {
//!         count, // ← field init shorthand
//!         other: 5
//!     })]
//!     test: Child,
//!
//!     #[br(args { count: 3 })]
//!     test2: Child,
//! }
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::prelude::*;
//! #[derive(BinWrite)]
//! #[bw(import {
//!     count: u32,
//!     other: u16 = 0 // ← optional argument
//! })]
//! struct Child {
//!     // ...
//! }
//!
//! #[derive(BinWrite)]
//! struct Parent {
//!     count: u32,
//!
//!     #[bw(args {
//!         count: *count,
//!         other: 5
//!     })]
//!     test: Child,
//!
//!     #[bw(args { count: 3 })]
//!     test2: Child,
//! }
//! ```
//! </div>
//!
//! ### Raw arguments
//!
//! Raw arguments allow the
//! <span class="br">[`Args`](crate::BinRead::Args)</span>
//! <span class="bw">[`Args`](crate::BinWrite::Args)</span>
//! type to be specified explicitly and to receive all arguments into a single
//! variable:
//!
//! <div class="br">
//!
//! ```text
//! #[br(import_raw($binding:ident : $ty:ty))]
//! #[br(args_raw($value:expr))] or #[br(args_raw = $value:expr)]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(import_raw($binding:ident : $ty:ty))]
//! #[bw(args_raw($value:expr))] or #[bw(args_raw = $value:expr)]
//! ```
//! </div>
//!
//! They are most useful for argument forwarding:
//!
//! <div class="br">
//!
//! ```
//! # use binrw::prelude::*;
//! type Args = (u32, u16);
//!
//! #[derive(BinRead)]
//! #[br(import_raw(args: Args))]
//! struct Child {
//!     // ...
//! }
//!
//! #[derive(BinRead)]
//! #[br(import_raw(args: Args))]
//! struct Middle {
//!     #[br(args_raw = args)]
//!     test: Child,
//! }
//!
//! #[derive(BinRead)]
//! struct Parent {
//!     count: u32,
//!
//!     #[br(args(1, 2))]
//!     mid: Middle,
//!
//!     // identical to `mid`
//!     #[br(args_raw = (1, 2))]
//!     mid2: Middle,
//! }
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::prelude::*;
//! type Args = (u32, u16);
//!
//! #[derive(BinWrite)]
//! #[bw(import_raw(args: Args))]
//! struct Child {
//!     // ...
//! }
//!
//! #[derive(BinWrite)]
//! #[bw(import_raw(args: Args))]
//! struct Middle {
//!     #[bw(args_raw = args)]
//!     test: Child,
//! }
//!
//! #[derive(BinWrite)]
//! struct Parent {
//!     count: u32,
//!
//!     #[bw(args(1, 2))]
//!     mid: Middle,
//!
//!     // identical to `mid`
//!     #[bw(args_raw = (1, 2))]
//!     mid2: Middle,
//! }
//! ```
//! </div>
//!
//! ## Limitations
//!
//! ### Borrowing values
//!
//! Non-static lifetimes for borrowed values are currently unavailable because
//! the associated type would require
//! [GATs](https://rust-lang.github.io/rfcs/1598-generic_associated_types.html)
//! to properly bind lifetimes to the function.
//!
//! ### Named arguments conflicting with `count` directive
//!
//! The [`count`](#count) directive may conflict with `args`. To pass arguments
//! to a type inside a [`Vec`], manually specify the count via named arguments
//! instead of using the `count` directive. See [`VecArgs`](crate::VecArgs) for
//! details.
//!
//! # Assert
//!
//! The `assert` directive validates objects and fields
//! <span class="br">after they are read,</span>
//! <span class="bw">before they are written,</span>
//! returning an error if the assertion condition evaluates to `false`:
//!
//! <div class="br">
//!
//! ```text
//! #[br(assert($cond:expr $(,)?))]
//! #[br(assert($cond:expr, $msg:literal $(,)?)]
//! #[br(assert($cond:expr, $fmt:literal, $($arg:expr),* $(,)?))]
//! #[br(assert($cond:expr, $err:expr $(,)?)]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(assert($cond:expr $(,)?))]
//! #[bw(assert($cond:expr, $msg:literal $(,)?)]
//! #[bw(assert($cond:expr, $fmt:literal, $($arg:expr),* $(,)?))]
//! #[bw(assert($cond:expr, $err:expr $(,)?)]
//! ```
//! </div>
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
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug)]
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
//! assert!(matches!(error, binrw::Error::AssertFail { message: expected, .. }));
//! ```
//! </div>
//! <div class="bw">
//!
//! TODO!
//! </div>
//!
//! ### Custom error
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(Debug, PartialEq)]
//! struct NotSmallerError(u32, u32);
//! impl core::fmt::Display for NotSmallerError {
//!     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//!         write!(f, "{} <= {}", self.0, self.1)
//!     }
//! }
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
//! </div>
//! <div class="bw">
//!
//! TODO!
//! </div>
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
//! In all cases, the <span class="br">reader</span><span class="bw">writer</span>’s
//! position is reset to where it was before
//! <span class="br">parsing</span><span class="bw">serialisation</span>
//! started.
//!
//! <div class="br">
//!
//! # Backtrace
//!
//! When an error is raised during parsing, BinRead forms a backtrace, bubbling the
//! error upwards and attaching additional information (surrounding code, line numbers,
//! messages, etc.) in order to aid in debugging.
//!
//! The `#[br(err_context(...))]` attribute can work in one of two ways:
//!
//! 1. If the first (or only) item is a string literal, it will be a message format string,
//! with any other arguments being used as arguments. This uses the same formatting as `format!`,
//! `println!`, and other standard library formatters.
//!
//! 2. Otherwise, only a single argument is allowed, which will then be attached as a context
//! type. This type must implement [`Display`](std::fmt::Display), [`Debug`], [`Send`], and [`Sync`].
//!
//! ## Example
//!
//! ```
//! # use binrw::{io::Cursor, BinRead, BinReaderExt};
//! #[derive(BinRead)]
//! struct InnerMostStruct {
//!     #[br(little)]
//!     len: u32,
//!
//!     #[br(count = len, err_context("len = {}", len))]
//!     items: Vec<u32>,
//! }
//!
//! #[derive(BinRead)]
//! struct MiddleStruct {
//!     #[br(little)]
//!     #[br(err_context("While parsing the innerest most struct"))]
//!     inner: InnerMostStruct,
//! }
//!
//! #[derive(Debug, Clone)] // Display implementation omitted
//! struct Oops(u32);
//!
//! #[derive(BinRead)]
//! struct OutermostStruct {
//!     #[br(little, err_context(Oops(3 + 1)))]
//!     middle: MiddleStruct,
//! }
//! # let mut x = Cursor::new(b"\0\0\0\x06");
//! # let err = x.read_be::<OutermostStruct>().map(|_| ()).unwrap_err();
//! # impl core::fmt::Display for Oops {
//! #     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//! #         write!(f, "Oops({})", self.0)
//! #     }
//! # }
//! ```
//! </div>
//!
//! # Byte order
//!
//! The `big` and `little` directives specify the [byte order](https://en.wikipedia.org/wiki/Endianness)
//! of data in a struct, enum, variant, or field:
//!
//! <div class="br">
//!
//! ```text
//! #[br(big)]
//! #[br(little)]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(big)]
//! #[bw(little)]
//! ```
//! </div>
//!
//! The `is_big` and `is_little` directives conditionally set the byte order of
//! a struct field:
//!
//! <div class="br">
//!
//! ```text
//! #[br(is_little = $cond:expr)] or #[br(is_little($cond:expr))]
//! #[br(is_big = $cond:expr)] or #[br(is_big($cond:expr))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(is_little = $cond:expr)] or #[bw(is_little($cond:expr))]
//! #[bw(is_big = $cond:expr)] or #[bw(is_big($cond:expr))]
//! ```
//! </div>
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
//! 4. <span class="br">The [`endian`](crate::ReadOptions::endian) property of the
//!    [`ReadOptions`](crate::ReadOptions) object passed to
//!    [`BinRead::read_options`](crate::BinRead::read_options) by the caller</span>
//!    <span class="bw">The [`endian`](crate::WriteOptions::endian) property of the
//!    [`WriteOptions`](crate::WriteOptions) object passed to
//!    [`BinWrite::write_options`](crate::BinWrite::write_options) by the caller</span>
//! 5. The host machine’s native byte order
//!
//! However, if a byte order directive is added to a struct or enum, that byte
//! order will *always* be used, even if the object is embedded in another
//! object or explicitly called with a different byte order:
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{Endian, ReadOptions, prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
//! #[br(little)] // ← this *forces* the struct to be little-endian
//! struct Child(u32);
//!
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
//! struct Parent {
//!     #[br(big)] // ← this will be ignored
//!     child: Child,
//! };
//!
//! let mut options = ReadOptions::new(Endian::Big /* ← this will be ignored */);
//! # assert_eq!(
//! Parent::read_options(&mut Cursor::new(b"\x01\0\0\0"), &options, ())
//! # .unwrap(), Parent { child: Child(1) });
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{Endian, WriteOptions, prelude::*, io::Cursor};
//! #[derive(BinWrite)]
//! # #[derive(Debug, PartialEq)]
//! #[bw(little)] // ← this *forces* the struct to be little-endian
//! struct Child(u32);
//!
//! #[derive(BinWrite)]
//! # #[derive(Debug, PartialEq)]
//! struct Parent {
//!     #[bw(big)] // ← this will be ignored
//!     child: Child,
//! };
//!
//! let object = Parent { child: Child(1) };
//!
//! let mut options = WriteOptions::new(Endian::Big /* ← this will be ignored */);
//! let mut output = Cursor::new(vec![]);
//! object.write_options(&mut output, &options, ())
//! # .unwrap();
//! # assert_eq!(output.into_inner(), b"\x01\0\0\0");
//! ```
//! </div>
//!
//! <span class="br">When manually implementing
//! [`BinRead::read_options`](crate::BinRead::read_options) or a
//! [custom parser function](#custom-parserswriters), the byte order is accessible
//! from [`ReadOptions::endian`](crate::ReadOptions::endian).</span>
//! <span class="bw">When manually implementing
//! [`BinWrite::write_options`](crate::BinWrite::write_options) or a
//! [custom writer function](#custom-parserswriters), the byte order is accessible
//! from [`WriteOptions::endian`](crate::WriteOptions::endian).</span>
//!
//! ## Examples
//!
//! ### Mixed endianness in one object
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
//! #[br(little)]
//! struct MyType (
//!     #[br(big)] u32, // ← will be big-endian
//!     u32, // ← will be little-endian
//! );
//!
//! # assert_eq!(MyType::read(&mut Cursor::new(b"\0\0\0\x01\x01\0\0\0")).unwrap(), MyType(1, 1));
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinWrite)]
//! #[bw(little)]
//! struct MyType (
//!     #[bw(big)] u32, // ← will be big-endian
//!     u32, // ← will be little-endian
//! );
//!
//! # let object = MyType(1, 1);
//! # let mut output = Cursor::new(vec![]);
//! # object.write_to(&mut output).unwrap();
//! # assert_eq!(output.into_inner(), b"\0\0\0\x01\x01\0\0\0");
//! ```
//! </div>
//!
//! ### Conditional field endianness
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! # #[derive(Debug, PartialEq)]
//! #[derive(BinRead)]
//! #[br(big)]
//! struct MyType {
//!     val: u8,
//!     #[br(is_little = (val == 3))]
//!     other_val: u16 // ← little-endian if `val == 3`, otherwise big-endian
//! }
//!
//! # assert_eq!(
//! MyType::read(&mut Cursor::new(b"\x03\x01\x00"))
//! # .unwrap(), MyType { val: 3, other_val: 1 });
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! # #[derive(Debug, PartialEq)]
//! #[derive(BinWrite)]
//! #[bw(big)]
//! struct MyType {
//!     val: u8,
//!     #[bw(is_little = (*val == 3))]
//!     other_val: u16 // ← little-endian if `val == 3`, otherwise big-endian
//! }
//!
//! let object = MyType { val: 3, other_val: 1 };
//! let mut output = Cursor::new(vec![]);
//! object.write_to(&mut output)
//! # .unwrap();
//! # assert_eq!(output.into_inner(), b"\x03\x01\x00");
//! ```
//! </div>
//!
//! # Calculations
//!
//! <div class="bw">
//!
//! **This directive can only be used with [`macro@binwrite`]. It will not work
//! with `#[derive(BinWrite)]`.**
//! </div>
//!
//! The `calc` directive computes the value of a field
//! <span class="br">instead of reading data from the reader:</span>
//! <span class="bw">to use when writing to the writer:</span>
//!
//! <div class="br">
//!
//! ```text
//! #[br(calc = $value:expr)] or #[br(calc($value:expr))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(calc = $value:expr)] or #[bw(calc($value:expr))]
//! ```
//! </div>
//!
//! Any <span class="br">earlier</span> field or [import](#arguments) can be
//! referenced by the expression in the directive.
//!
//! <div class="bw">
//!
//! Since the field is treated as a temporary variable instead of an actual
//! field, when deriving `BinRead`, the field should also be annotated with
//! `#[br(temp)]`.
//! </div>
//!
//! ## Examples
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct MyType {
//!     var: u32,
//!     #[br(calc = 3 + var)]
//!     var_plus_3: u32,
//! }
//!
//! # assert_eq!(Cursor::new(b"\0\0\0\x01").read_be::<MyType>().unwrap().var_plus_3, 4);
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[binwrite]
//! #[bw(big)]
//! struct MyType {
//!     var: u32,
//!     #[bw(calc = var - 3)]
//!     var_minus_3: u32,
//! }
//!
//! let object = MyType { var: 4 };
//!
//! let mut output = Cursor::new(vec![]);
//! object.write_to(&mut output).unwrap();
//! assert_eq!(output.into_inner(), b"\0\0\0\x04\0\0\0\x01");
//! ```
//! </div>
//!
//! <div class="br">
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
//! ```
//! # use binrw::{prelude::*, io::Cursor};
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
//! ```
//! # use binrw::{prelude::*, io::Cursor};
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
//! </div>
//!
//! <div class="br">
//!
//! # Count
//!
//! The `count` directive is a shorthand for passing a `count` argument to a
//! parser like [`Vec`]:
//!
//! ```text
//! #[br(count = $count:expr) or #[br(count($count:expr))]
//! ```
//!
//! It desugars to:
//!
//! ```text
//! #[br(args { count: $count as usize })]
//! ```
//!
//! When manually implementing
//! [`BinRead::read_options`](crate::BinRead::read_options) or a
//! [custom parser function](#custom-parserswriters), the `count` value is accessible
//! from a named argument named `count`.
//!
//! Any earlier field or [import](#arguments) can be referenced by the
//! expression in the directive.
//!
//! ## Examples
//!
//! ### Using `count` with [`Vec`]
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
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
//! </div>
//!
//! # Custom <span class="br">parsers</span><span class="bw">writers</span>
//!
//! <div class="br">
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
//! </div>
//! <div class="bw">
//!
//! The `write_with` directive specifies a custom serialisation function which
//! can be used to override the default [`BinWrite`](crate::BinWrite)
//! implementation for a type, or to serialise types which have no `BinWrite`
//! implementation at all:
//!
//! ```text
//! #[bw(write_with = $write_fn:expr)] or #[bw(write_with($write_fn:expr))]
//! ```
//!
//! Any field or [import](#arguments) can be referenced by the expression in the
//! directive (for example, to construct a serialisation function at runtime by
//! calling a function generator).
//! </div>
//!
//! ## Examples
//!
//! <div class="br">
//!
//! ### Using a custom parser to generate a [`HashMap`](std::collections::HashMap)
//!
//! ```
//! # use binrw::{prelude::*, io::*, ReadOptions};
//! # use std::collections::HashMap;
//! fn custom_parser<R: Read + Seek>(reader: &mut R, ro: &ReadOptions, _: ())
//!     -> BinResult<HashMap<u16, u16>>
//! {
//!     let mut map = HashMap::new();
//!     map.insert(
//!         <_>::read_options(reader, ro, ())?,
//!         <_>::read_options(reader, ro, ())?,
//!     );
//!     Ok(map)
//! }
//!
//! #[derive(BinRead)]
//! #[br(big)]
//! struct MyType {
//!     #[br(parse_with = custom_parser)]
//!     offsets: HashMap<u16, u16>
//! }
//!
//! # assert_eq!(Cursor::new(b"\0\0\0\x01").read_be::<MyType>().unwrap().offsets.get(&0), Some(&1));
//! ```
//! </div>
//! <div class="bw">
//!
//! ### Using a custom serialiser to write a [`BTreeMap`](std::collections::BTreeMap)
//!
//! ```
//! # use binrw::{prelude::*, io::*, WriteOptions};
//! # use std::collections::BTreeMap;
//! fn custom_writer<R: Write + Seek>(
//!     map: &BTreeMap<u16, u16>,
//!     writer: &mut R,
//!     wo: &WriteOptions,
//!     _: ()
//! ) -> BinResult<()> {
//!     for (key, val) in map.iter() {
//!         key.write_options(writer, wo, ())?;
//!         val.write_options(writer, wo, ())?;
//!     }
//!     Ok(())
//! }
//!
//! #[derive(BinWrite)]
//! #[bw(big)]
//! struct MyType {
//!     #[bw(write_with = custom_writer)]
//!     offsets: BTreeMap<u16, u16>
//! }
//!
//! let object = MyType {
//!     offsets: BTreeMap::from([(0, 1), (2, 3)]),
//! };
//!
//! let mut output = Cursor::new(vec![]);
//! object.write_to(&mut output).unwrap();
//! assert_eq!(output.into_inner(), b"\0\0\0\x01\0\x02\0\x03");
//! ```
//! </div>
//!
//! <div class="br">
//!
//! ### Using `FilePtr::parse` to read a `NullString` without storing a `FilePtr`
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor, FilePtr32, NullString};
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(parse_with = FilePtr32::parse)]
//!     some_string: NullString,
//! }
//!
//! # let val: MyType = Cursor::new(b"\0\0\0\x04Test\0").read_be().unwrap();
//! # assert_eq!(val.some_string.to_string(), "Test");
//! ```
//! </div>
//! <div class="bw">
//!
//! TODO!
//! </div>
//!
//! <div class="br">
//!
//! # Enum errors
//!
//! The `return_all_errors` (default) and `return_unexpected_error` directives
//! define how to handle errors when parsing an enum:
//!
//! ```text
//! #[br(return_all_errors)]
//! #[br(return_unexpected_error)]
//! ```
//!
//! `return_all_errors` collects the errors that occur when enum variants fail
//! to parse and returns them in [`binrw::Error::EnumErrors`] when no variants
//! parse successfully:
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug)]
//! #[br(return_all_errors)]
//! enum Test {
//!     #[br(magic(0u8))]
//!     A { a: u8 },
//!     B { b: u32 },
//!     C { #[br(assert(c != 1))] c: u8 },
//! }
//!
//! let error = Test::read(&mut Cursor::new(b"\x01")).unwrap_err();
//! if let binrw::Error::EnumErrors { pos, variant_errors } = error {
//!     assert_eq!(pos, 0);
//!     assert!(matches!(variant_errors[0], ("A", binrw::Error::BadMagic { .. })));
//!     assert!(matches!(
//!         (variant_errors[1].0, variant_errors[1].1.root_cause()),
//!         ("B", binrw::Error::Io(..))
//!     ));
//!     assert!(matches!(variant_errors[2], ("C", binrw::Error::AssertFail { .. })));
//! }
//! # else {
//! #    panic!("wrong error type");
//! # }
//! ```
//!
//! `return_unexpected_error` discards the errors and instead returns a generic
//! [`binrw::Error::NoVariantMatch`] if all variants fail to parse. This avoids
//! extra memory allocations required to collect errors, but only provides the
//! position when parsing fails:
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug)]
//! #[br(return_unexpected_error)]
//! enum Test {
//!     #[br(magic(0u8))]
//!     A { a: u8 },
//!     B { b: u32 },
//!     C { #[br(assert(c != 1))] c: u8 },
//! }
//!
//! let error = Test::read(&mut Cursor::new(b"\x01")).unwrap_err();
//! if let binrw::Error::NoVariantMatch { pos } = error {
//!     assert_eq!(pos, 0);
//! }
//! # else {
//! #    panic!("wrong error type");
//! # }
//! ```
//!
//! </div>
//!
//! # Ignore
//!
//! <div class="br">
//!
//! The `ignore` directive, and its alias `default`, sets the value of the field
//! to its [`Default`](core::default::Default) instead of reading data from the
//! reader:
//!
//! <div class="br">
//!
//! ```text
//! #[br(default)] or #[br(ignore)]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(ignore)]
//! ```
//! </div>
//!
//! ## Examples
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
//! struct Test {
//!     #[br(ignore)]
//!     path: Option<std::path::PathBuf>,
//! }
//!
//! assert_eq!(
//!     Test::read(&mut Cursor::new(b"")).unwrap(),
//!     Test { path: None }
//! );
//! ```
//! </div>
//! <div class="bw">
//!
//! The `ignore` directive skips writing the field to the writer:
//!
//! ```text
//! #[br(ignore)]
//! ```
//!
//! ## Examples
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinWrite)]
//! struct Test {
//!     a: u8,
//!     #[bw(ignore)]
//!     b: u8,
//!     c: u8,
//! }
//!
//! let object = Test { a: 1, b: 2, c: 3 };
//! let mut output = Cursor::new(vec![]);
//! object.write_to(&mut output).unwrap();
//! assert_eq!(
//!     output.into_inner(),
//!     b"\x01\x03"
//! );
//! ```
//! </div>
//!
//! # Magic
//!
//! The `magic` directive matches [magic numbers](https://en.wikipedia.org/wiki/Magic_number_(programming))
//! in data:
//!
//! <div class="br">
//!
//! ```text
//! #[br(magic = $magic:literal)] or #[br(magic($magic:literal))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(magic = $magic:literal)] or #[bw(magic($magic:literal))]
//! ```
//! </div>
//!
//! The magic number can be a byte literal, byte string, float, or integer. When
//! a magic number is matched, parsing begins with the first byte after the
//! magic number in the data. When a magic number is not matched, an error is
//! returned.
//!
//! ## Examples
//!
//! ### Using byte strings
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
//! #[br(magic = b"TEST")]
//! struct Test {
//!     val: u32
//! }
//!
//! # assert_eq!(
//! Test::read(&mut Cursor::new(b"TEST\0\0\0\0"))
//! # .unwrap(), Test { val: 0 });
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinWrite)]
//! # #[derive(Debug, PartialEq)]
//! #[bw(magic = b"TEST")]
//! struct Test {
//!     val: u32
//! }
//!
//! let object = Test { val: 0 };
//! let mut output = Cursor::new(vec![]);
//! object.write_to(&mut output)
//! # .unwrap();
//! # assert_eq!(output.into_inner(), b"TEST\0\0\0\0");
//! ```
//! </div>
//!
//! ### Using float literals
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
//! #[br(big, magic = 1.2f32)]
//! struct Version(u16);
//!
//! # assert_eq!(
//! Version::read(&mut Cursor::new(b"\x3f\x99\x99\x9a\0\0"))
//! # .unwrap(), Version(0));
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinWrite)]
//! # #[derive(Debug, PartialEq)]
//! #[bw(big, magic = 1.2f32)]
//! struct Version(u16);
//!
//! let object = Version(0);
//! let mut output = Cursor::new(vec![]);
//! object.write_to(&mut output)
//! # .unwrap();
//! # assert_eq!(output.into_inner(), b"\x3f\x99\x99\x9a\0\0");
//! ```
//! </div>
//!
//! ### Enum variant selection using magic
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
//! enum Command {
//!     #[br(magic = 0u8)] Nop,
//!     #[br(magic = 1u8)] Jump { loc: u32 },
//!     #[br(magic = 2u8)] Begin { var_count: u16, local_count: u16 }
//! }
//!
//! # assert_eq!(
//! Command::read(&mut Cursor::new(b"\x01\0\0\0\0"))
//! # .unwrap(), Command::Jump { loc: 0 });
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinWrite)]
//! # #[derive(Debug, PartialEq)]
//! enum Command {
//!     #[bw(magic = 0u8)] Nop,
//!     #[bw(magic = 1u8)] Jump { loc: u32 },
//!     #[bw(magic = 2u8)] Begin { var_count: u16, local_count: u16 }
//! }
//!
//! let object = Command::Jump { loc: 0 };
//! let mut output = Cursor::new(vec![]);
//! object.write_to(&mut output)
//! # .unwrap();
//! # assert_eq!(output.into_inner(), b"\x01\0\0\0\0");
//! ```
//! </div>
//!
//! <div class="br">
//!
//! ## Errors
//!
//! If the specified magic number does not match the data, a
//! [`BadMagic`](crate::Error::BadMagic) error is returned and the reader’s
//! position is reset to where it was before parsing started.
//! </div>
//!
//! # Map
//!
//! The `map` and `try_map` directives allow data to be read using one type and
//! stored as another:
//!
//! <div class="br">
//!
//! ```text
//! #[br(map = $map_fn:expr)] or #[br(map($map_fn:expr)))]
//! #[br(try_map = $map_fn:expr)] or #[br(try_map($map_fn:expr)))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(map = $map_fn:expr)] or #[bw(map($map_fn:expr)))]
//! #[bw(try_map = $map_fn:expr)] or #[bw(try_map($map_fn:expr)))]
//! ```
//! </div>
//!
//! <span class="br">When using `map` on a field, the map function must
//! explicitly declare the type of the data to be read in its first parameter
//! and return a value which matches the type of the field.</span>
//! <span class="bw">When using `map` on a field, the map function will receive
//! an immutable reference to the field value and must return a type which
//! implements [`BinWrite`](binrw::BinWrite).</span> The map function can be a
//! plain function, closure, or call expression which returns a plain function
//! or closure.
//!
//! When using `try_map`, the same rules apply, except that the function must
//! return a [`Result<T, E>`](Result) instead.
//!
//! When using `map` or `try_map` on a struct or enum, the map function
//! <span class="br">must return `Self` or `Result<Self, E>`.</span>
//! <span class="bw">will receive an immutable reference to the entire object
//! and must return a type that implements [`BinWrite`](binrw::BinWrite).</span>
//!
//! Any <span class="br">earlier</span> field or [import](#arguments) can be
//! referenced by the expression in the directive.
//!
//! ## Examples
//!
//! ### Using `map` on a field
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(map = |x: u8| x.to_string())]
//!     int_str: String
//! }
//!
//! # assert_eq!(Cursor::new(b"\0").read_be::<MyType>().unwrap().int_str, "0");
//! ```
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinWrite)]
//! struct MyType {
//!     #[bw(map = |x| x.parse::<u8>().unwrap())]
//!     int_str: String
//! }
//!
//! let object = MyType { int_str: String::from("1") };
//! let mut output = Cursor::new(vec![]);
//! object.write_to(&mut output).unwrap();
//! assert_eq!(output.into_inner(), b"\x01");
//! ```
//! </div>
//!
//! ### Using `try_map` on a field
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
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
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! # use core::convert::TryFrom;
//! #[derive(BinWrite)]
//! struct MyType {
//!     #[bw(try_map = |x| { i8::try_from(*x) })]
//!     value: u8
//! }
//!
//! let mut writer = Cursor::new(Vec::new());
//! writer.write_be(&MyType { value: 3 });
//! assert_eq!(&writer.into_inner()[..], b"\x03")
//! ```
//! </div>
//!
//! ### Using `map` on a struct to create a bit field
//!
//! The [`modular-bitfield`](https://docs.rs/modular-bitfield) crate can be used
//! along with `map` to create a struct out of raw bits.
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
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
//! </div>
//! <div class="bw">
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
//! # let mut output = Cursor::new(vec![]);
//! # output.write_le(&PackedData::new().with_status(3)).unwrap();
//! # assert_eq!(output.into_inner(), b"\x03");
//! ```
//! </div>
//!
//! ## Errors
//!
//! If the `try_map` function returns a [`binrw::io::Error`](crate::io::Error)
//! or [`std::io::Error`], an [`Io`](crate::Error::Io) error is returned. For
//! any other error type, a [`Custom`](crate::Error::Custom) error is returned.
//!
//! In all cases, the
//! <span class="br">reader’s</span><span class="bw">writer’s</span> position is
//! reset to where it was before parsing started.
//!
//! <div class="br">
//!
//! # Offset
//!
//! The `offset` and `offset_after` directives are shorthands for passing
//! `offset` and `offset_after` arguments to a parser that operates like
//! [`FilePtr`](crate::FilePtr):
//!
//! ```text
//! #[br(offset = $offset:expr)] or #[br(offset($offset:expr))]
//! #[br(offset_after = $offset:expr)] or #[br(offset_after($offset:expr))]
//! ```
//!
//! When manually implementing
//! [`BinRead::read_options`](crate::BinRead::read_options) or a
//! [custom parser function](#custom-parserswriters), the offset is accessible
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
//! ```
//! # use binrw::{prelude::*, io::Cursor, FilePtr};
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
//! </div>
//!
//! # Padding and alignment
//!
//! binrw includes directives for common forms of
//! [data structure alignment](https://en.wikipedia.org/wiki/Data_structure_alignment#Data_structure_padding).
//!
//! The `pad_before` and `pad_after` directives skip a specific number of bytes
//! either before or after
//! <span class="br">reading</span><span class="bw">writing</span> a field,
//! respectively:
//!
//! <div class="br">
//!
//! ```text
//! #[br(pad_after = $skip_bytes:expr)] or #[br(pad_after($skip_bytes:expr))]
//! #[br(pad_before = $skip_bytes:expr)] or #[br(pad_before($skip_bytes:expr))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(pad_after = $skip_bytes:expr)] or #[bw(pad_after($skip_bytes:expr))]
//! #[bw(pad_before = $skip_bytes:expr)] or #[bw(pad_before($skip_bytes:expr))]
//! ```
//! </div>
//!
//! This is equivalent to:
//!
//! ```
//! # let mut pos = 0;
//! # let padding = 0;
//! pos += padding;
//! ```
//!
//! ---
//!
//! The `align_before` and `align_after` directives align the next
//! <span class="br">read</span><span class="bw">write</span> to the
//! given byte alignment either before or after
//! <span class="br">reading</span><span class="bw">writing</span> a field,
//! respectively:
//!
//! <div class="br">
//! ```text
//! #[br(align_after = $align_to:expr)] or #[br(align_after($align_to:expr))]
//! #[br(align_before = $align_to:expr)] or #[br(align_before($align_to:expr))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(align_after = $align_to:expr)] or #[bw(align_after($align_to:expr))]
//! #[bw(align_before = $align_to:expr)] or #[bw(align_before($align_to:expr))]
//! ```
//! </div>
//!
//! This is equivalent to:
//!
//! ```
//! # let mut pos = 0;
//! # let align = 1;
//! if pos % align != 0 {
//!     pos += align - (pos % align);
//! }
//! ```
//!
//! ---
//!
//! The `seek_before` directive accepts a [`SeekFrom`](crate::io::SeekFrom)
//! object and seeks the
//! <span class="br">reader</span><span class="bw">writer</span> to an arbitrary
//! position before
//! <span class="br">reading</span><span class="bw">writing</span> a field:
//!
//! <div class="br">
//!
//! ```text
//! #[br(seek_before = $seek_from:expr)] or #[br(seek_before($seek_from:expr))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(seek_before = $seek_from:expr)] or #[bw(seek_before($seek_from:expr))]
//! ```
//! </div>
//!
//! This is equivalent to:
//!
//! ```
//! # use binrw::io::Seek;
//! # let mut stream = binrw::io::Cursor::new(vec![]);
//! # let seek_from = binrw::io::SeekFrom::Start(0);
//! stream.seek(seek_from)?;
//! # Ok::<(), binrw::io::Error>(())
//! ```
//!
//! The position of the
//! <span class="br">reader</span><span class="bw">writer</span> will not be
//! restored after the seek; use the
//! [`restore_position`](#restore-position) directive to seek, then
//! <span class="br">read</span><span class="bw">write</span>, then restore
//! position.
//!
//! ---
//!
//! The `pad_size_to` directive will ensure that the
//! <span class="br">reader</span><span class="bw">writer</span> has advanced at
//! least the number of bytes given after the field has been
//! <span class="br">read</span><span class="bw">written</span>:
//!
//! <div class="br">
//!
//! ```text
//! #[br(pad_size_to = $size:expr)] or #[br(pad_size_to($size:expr))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(pad_size_to = $size:expr)] or #[bw(pad_size_to($size:expr))]
//! ```
//! </div>
//!
//! For example, if a format uses a null-terminated string, but always reserves
//! at least 256 bytes for that string, [`NullString`](crate::NullString) will
//! read the string and `pad_size_to(256)` will ensure the reader skips whatever
//! padding, if any, remains. If the string is longer than 256 bytes, no padding
//! will be skipped.
//!
//! Any <span class="br">earlier</span> field or [import](#arguments) can be
//! referenced by the expressions in any of these directives.
//!
//! ## Examples
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, NullString, io::SeekFrom};
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
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, NullString, io::SeekFrom};
//! #[derive(BinWrite)]
//! struct MyType {
//!     #[bw(align_before = 4, pad_after = 1, align_after = 4)]
//!     str: NullString,
//!
//!     #[bw(pad_size_to = 0x10)]
//!     test: u64,
//!
//!     #[bw(seek_before = SeekFrom::End(-4))]
//!     end: u32,
//! }
//! ```
//! </div>
//!
//! ## Errors
//!
//! If seeking fails, an [`Io`](crate::Error::Io) error is returned and the
//! <span class="br">reader’s</span><span class="bw">writer’s</span> position is
//! reset to where it was before
//! <span class="br">parsing</span><span class="bw">serialisation</span>
//! started.
//!
//! <div class="br">
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
//! # use binrw::{prelude::*, FilePtr32, NullString, io::Cursor};
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
//! </div>
//!
//! <div class="br">
//!
//! # Pre-assert
//!
//! `pre_assert` works like [`assert`](#assert), but checks the condition before
//! data is read instead of after:
//!
//! ```text
//! #[br(pre_assert($cond:expr $(,)?))]
//! #[br(pre_assert($cond:expr, $msg:literal $(,)?)]
//! #[br(pre_assert($cond:expr, $fmt:literal, $($arg:expr),* $(,)?))]
//! #[br(pre_assert($cond:expr, $err:expr $(,)?)]
//! ```
//!
//! This is most useful when validating arguments or selecting an enum variant.
//!
//! ## Examples
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
//! #[br(import { ty: u8 })]
//! enum Command {
//!     #[br(pre_assert(ty == 0))] Variant0(u16, u16),
//!     #[br(pre_assert(ty == 1))] Variant1(u32)
//! }
//!
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
//! struct Message {
//!     ty: u8,
//!     len: u8,
//!     #[br(args { ty })]
//!     data: Command
//! }
//!
//! let msg = Cursor::new(b"\x01\x04\0\0\0\xFF").read_be::<Message>();
//! assert!(msg.is_ok());
//! let msg = msg.unwrap();
//! assert_eq!(msg, Message { ty: 1, len: 4, data: Command::Variant1(0xFF) });
//! ```
//! </div>
//!
//! # Repr
//!
//! The `repr` directive is used on a unit-like (C-style) enum to specify the
//! underlying type to use when
//! <span class="br">reading</span><span class="bw">writing</span> the
//! field<span class="br"> and matching variants</span>:
//!
//! <div class="br">
//!
//! ```text
//! #[br(repr = $ty:ty)] or #[br(repr($ty:ty))]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(repr = $ty:ty)] or #[bw(repr($ty:ty))]
//! ```
//! </div>
//!
//! ## Examples
//!
//! <div class="br">
//!
//! ```
//! # use binrw::BinRead;
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
//! </div>
//! <div class="bw">
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
//! </div>
//!
//! ## Errors
//!
//! If a <span class="br">read</span><span class="bw">write</span> fails, an
//! [`Io`](crate::Error::Io) error is returned. <span class="br">If no variant
//! matches, a [`NoVariantMatch`](crate::Error::NoVariantMatch) error is
//! returned.</span>
//!
//! In all cases, the
//! <span class="br">reader’s</span><span class="bw">writer’s</span> position is
//! reset to where it was before
//! <span class="br">parsing</span><span class="bw">serialisation</span>
//! started.
//!
//! # Restore position
//!
//! The `restore_position` directive restores the position of the
//! <span class="br">reader</span><span class="bw">writer</span> after a field
//! is <span class="br">read</span><span class="bw">written</span>:
//!
//! <div class="br">
//!
//! ```text
//! #[br(restore_position)]
//! ```
//! </div>
//! <div class="bw">
//!
//! ```text
//! #[bw(restore_position)]
//! ```
//! </div>
//!
//! To seek to an arbitrary position, use [`seek_before`](#padding-and-alignment)
//! instead.
//!
//! ## Examples
//!
//! <div class="br">
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! # #[derive(Debug, PartialEq)]
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
//! </div>
//! <div class="bw">
//!
//! ```
//! # use binrw::{prelude::*, io::{Cursor, SeekFrom}};
//! #[derive(BinWrite)]
//! # #[derive(Debug, PartialEq)]
//! #[bw(big)]
//! struct Relocation {
//!     #[bw(ignore)]
//!     delta: u32,
//!     #[bw(seek_before(SeekFrom::Current((*delta).into())))]
//!     reloc: u32,
//! }
//!
//! #[derive(BinWrite)]
//! # #[derive(Debug, PartialEq)]
//! struct Executable {
//!     #[bw(restore_position)]
//!     code: Vec<u8>,
//!     relocations: Vec<Relocation>,
//! }
//!
//! let object = Executable {
//!     code: vec![ 1, 2, 3, 4, 0, 0, 0, 0, 9, 10, 0, 0, 0, 0 ],
//!     relocations: vec![
//!         Relocation { delta: 4, reloc: 84281096 },
//!         Relocation { delta: 2, reloc: 185339150 },
//!     ]
//! };
//! let mut output = Cursor::new(vec![]);
//! object.write_to(&mut output).unwrap();
//! assert_eq!(
//!   output.into_inner(),
//!   b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e"
//! );
//! ```
//! </div>
//!
//! ## Errors
//!
//! If querying or restoring the
//! <span class="br">reader</span><span class="bw">writer</span> position fails,
//! an [`Io`](crate::Error::Io) error is returned and the
//! <span class="br">reader’s</span><span class="bw">writer’s</span>
//! position is reset to where it was before
//! <span class="br">parsing</span><span class="bw">serialisation</span>
//! started.
//!
//! <div class="br">
//!
//! # Temp
//!
//! **This directive can only be used with [`macro@binread`]. It will not work
//! with `#[derive(BinRead)]`.**
//!
//! The `temp` directive causes a field to be treated as a temporary variable
//! instead of an actual field. The field will be removed from the struct
//! definition generated by [`macro@binread`]:
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
//! ```
//! # use binrw::{BinRead, io::Cursor, binread};
//! #[binread]
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
//! </div>
//!
//! <div class="br">
//!
//! # Try
//!
//! The `try` directive allows parsing of a field to fail instead
//! of returning an error:
//!
//! ```text
//! #[br(try)]
//! ```
//!
//! If the field cannot be parsed, the position of the reader will be restored
//! and the value of the field will be set to the [`default`](core::default::Default) value for the type.
//!
//! ## Examples
//!
//! ```
//! # use binrw::{prelude::*, io::Cursor};
//! #[derive(BinRead)]
//! struct MyType {
//!     #[br(try)]
//!     maybe_u32: Option<u32>
//! }
//!
//! assert_eq!(Cursor::new(b"").read_be::<MyType>().unwrap().maybe_u32, None);
//! ```
//! </div>

#![allow(unused_imports)]

#[cfg(all(doc, not(feature = "std")))]
extern crate std;
#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;

use crate::{binread, binwrite};
