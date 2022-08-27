//! Additional long-form documentation and reference material.

#[doc = include_str!("../doc/attribute.md")]
pub mod attribute {}
#[doc = include_str!("../doc/performance.md")]
pub mod performance {}

#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;
