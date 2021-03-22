//! Type definitions for byte order handling.

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Defines the order of bytes in a multi-byte type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Endian {
    /// The most significant byte is stored first.
    Big,
    /// The least significant byte is stored first.
    Little,
    /// The byte order is determined by the host CPU.
    Native,
}

pub use Endian::{Big as BE, Little as LE, Native as NE};

impl From<&Endian> for String {
    fn from(endian: &Endian) -> String {
        String::from(match endian {
            Endian::Big => "Big",
            Endian::Little => "Little",
            Endian::Native => "Native",
        })
    }
}

impl Default for Endian {
    fn default() -> Endian {
        Endian::Native
    }
}
