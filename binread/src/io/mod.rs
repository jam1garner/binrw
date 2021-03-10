//! A swappable version of [std::io](std::io) that works in `no_std + alloc` environments.
//! If the feature flag `std` is enabled (as it is by default), this will just re-export types from `std::io`.

pub mod prelude;
pub mod cursor;
pub mod error;

#[cfg(not(feature = "std"))]
mod no_std;
#[cfg(not(feature = "std"))]
pub use no_std::*;

#[cfg(feature = "std")]
pub use std::io::{Bytes, Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};
