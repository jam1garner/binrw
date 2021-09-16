#[cfg(all(doc, not(feature = "std")))]
extern crate alloc;
use super::Endian;
#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;

/// Runtime options for
/// [`BinRead::read_options()`](crate::BinRead::read_options).
#[derive(Default, Clone, Copy)]
pub struct ReadOptions {
    /// The [byte order](crate::Endian) to use when reading data.
    ///
    /// Note that if a derived type uses one of the
    /// [byte order directives](crate::attribute#byte-order), this option
    /// will be overridden by the directive.
    endian: Endian,

    /// An absolute offset added to the [`FilePtr::ptr`](crate::FilePtr::ptr)
    /// offset before reading the pointed-to value.
    offset: u64,
}

impl ReadOptions {
    /// Create a new ReadOptions with a given Endian
    pub fn new(endian: Endian) -> Self {
        Self {
            endian,
            ..Default::default()
        }
    }

    /// Returns the given ReadOptions but with the endian replaced
    pub fn with_endian(self, endian: Endian) -> Self {
        Self { endian, ..self }
    }

    /// The [byte order](crate::Endian) to use when reading data.
    ///
    /// Note that if a derived type uses one of the
    /// [byte order directives](crate::attribute#byte-order), this option
    /// will be overridden by the directive.
    pub fn endian(&self) -> Endian {
        self.endian
    }

    /// Returns the given ReadOptions but with the offset replaced
    pub fn with_offset(self, offset: u64) -> Self {
        Self { offset, ..self }
    }

    /// An absolute offset added to the [`FilePtr::ptr`](crate::FilePtr::ptr)
    /// offset before reading the pointed-to value.
    pub fn offset(&self) -> u64 {
        self.offset
    }
}
