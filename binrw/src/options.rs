#[cfg(all(doc, not(feature = "std")))]
extern crate alloc;

#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;

use crate::Endian;

/// Runtime options for
/// [`BinRead::read_options()`](crate::BinRead::read_options).
#[non_exhaustive]
#[derive(Default, Clone, Copy)]
pub struct ReadOptions {
    /// The [byte order](crate::Endian) to use when reading data.
    ///
    /// Note that if a derived type uses one of the
    /// [byte order directives](crate::attribute#byte-order), this option
    /// will be overridden by the directive.
    pub endian: Endian,

    /// An absolute offset added to the [`FilePtr::ptr`](crate::FilePtr::ptr)
    /// offset before reading the pointed-to value.
    pub offset: u64,
}

#[derive(Default, Clone, Copy)]
pub struct WriteOptions {
    endian: Endian,
}

impl WriteOptions {
    pub fn new() -> Self {
        Self {
            endian: Endian::Native
        }
    }

    pub fn with_endian(self, endian: Endian) -> Self {
        Self {
            endian,
            ..self
        }
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }
}
