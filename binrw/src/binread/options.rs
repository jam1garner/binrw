use super::Endian;
#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;

/// Runtime options for
/// [`BinRead::read_options()`](crate::BinRead::read_options).
#[derive(Clone, Copy)]
pub struct ReadOptions {
    /// The [byte order](crate::Endian) to use when reading data.
    ///
    /// Note that if a derived type uses one of the
    /// [byte order directives](crate::docs::attribute#byte-order), this option
    /// will be overridden by the directive.
    endian: Endian,

    /// An absolute offset added to the [`FilePtr::ptr`](crate::FilePtr::ptr)
    /// offset before reading the pointed-to value.
    offset: u64,
}

impl ReadOptions {
    /// Creates a new `ReadOptions` with the given [endianness](crate::Endian).
    #[must_use]
    pub fn new(endian: Endian) -> Self {
        Self {
            endian,
            offset: <_>::default(),
        }
    }

    /// The [byte order](crate::Endian) to use when reading data.
    ///
    /// Note that if a derived type uses one of the
    /// [byte order directives](crate::docs::attribute#byte-order), this option
    /// will be overridden by the directive.
    #[must_use]
    pub fn endian(&self) -> Endian {
        self.endian
    }

    /// An absolute offset added to the [`FilePtr::ptr`](crate::FilePtr::ptr)
    /// offset before reading the pointed-to value.
    #[must_use]
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Creates a copy of this `ReadOptions` using the given
    /// [endianness](crate::Endian).
    #[must_use]
    pub fn with_endian(self, endian: Endian) -> Self {
        Self { endian, ..self }
    }

    /// Creates a copy of this `ReadOptions` using the given
    /// [offset](crate::docs::attribute#offset).
    #[must_use]
    pub fn with_offset(self, offset: u64) -> Self {
        Self { offset, ..self }
    }
}
