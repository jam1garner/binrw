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
}

impl ReadOptions {
    /// Creates a new `ReadOptions` with the given [endianness](crate::Endian).
    #[must_use]
    pub fn new(endian: Endian) -> Self {
        Self { endian }
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

    /// Creates a copy of this `ReadOptions` using the given
    /// [endianness](crate::Endian).
    #[must_use]
    // Lint: API compatibility.
    #[allow(clippy::unused_self)]
    pub fn with_endian(self, endian: Endian) -> Self {
        Self { endian }
    }
}
