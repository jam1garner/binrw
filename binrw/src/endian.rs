//! Type definitions for byte order handling.

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

impl core::fmt::Display for Endian {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Big => write!(f, "Big"),
            Self::Little => write!(f, "Little"),
            Self::Native => write!(f, "Native"),
        }
    }
}

impl Default for Endian {
    fn default() -> Endian {
        Endian::Native
    }
}
