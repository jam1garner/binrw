#[cfg(all(doc, not(feature = "std")))]
extern crate alloc;
use super::Endian;
#[cfg(all(doc, not(feature = "std")))]
use alloc::vec::Vec;

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
