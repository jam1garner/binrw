//! Additional long-form documentation and reference material.

#[cfg(all(doc, not(feature = "std")))]
extern crate std;
#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;

#[doc = include_str!("../doc/attribute.md")]
pub mod attribute {}
#[doc = include_str!("../doc/performance.md")]
pub mod performance {}
