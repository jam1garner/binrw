//! Traits, helpers, and type definitions for core I/O functionality.
//!
//! By default, this module simply re-exports the parts of [`std::io`] that are
//! used by binrw. In no_std environments, a compatible subset API is exposed
//! instead.

#[cfg(feature = "std")]
mod bufreader;
pub mod prelude;
mod seek;
#[cfg(all(doc, not(feature = "std")))]
extern crate std;
#[cfg(not(feature = "std"))]
mod no_std;
#[cfg(feature = "std")]
pub use bufreader::BufReader;
#[cfg(not(feature = "std"))]
pub use no_std::*;
pub use seek::NoSeek;
#[cfg(feature = "std")]
pub use std::io::{Bytes, Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};
