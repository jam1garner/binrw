//! Traits, helpers, and type definitions for core I/O functionality.
//!
//! By default, this module simply re-exports the parts of [`std::io`] that are
//! used by binrw. In `no_std` environments, a compatible subset API is exposed
//! instead.

#[cfg(feature = "std")]
mod bufreader;
#[cfg(not(feature = "std"))]
mod no_std;
pub mod prelude;
mod seek;

#[cfg(feature = "std")]
pub use bufreader::BufReader;
#[cfg(all(doc, not(feature = "std")))]
#[doc(hidden)]
pub struct BufReader;
#[cfg(not(feature = "std"))]
pub use no_std::*;
pub use seek::NoSeek;
#[cfg(feature = "std")]
pub use std::io::{Bytes, Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
