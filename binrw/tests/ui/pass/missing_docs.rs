//! https://github.com/rust-lang/rust/issues/24584

#![deny(missing_docs)]

use binrw::binrw;

/// Foo.
#[binrw]
#[brw(import { _a: u32 })]
pub struct Foo {
    _a: u8,
}

/// Main.
fn main() {}
