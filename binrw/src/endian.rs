//! Type definitions for byte order handling.

use crate::BinResult;
use alloc::boxed::Box;
pub use Endian::{Big as BE, Little as LE};

/// Defines the order of bytes in a multi-byte type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Endian {
    /// The most significant byte is stored first.
    Big,
    /// The least significant byte is stored first.
    Little,
}

impl Endian {
    #[cfg(target_endian = "big")]
    /// The target platform’s native endianness.
    pub const NATIVE: Self = Endian::Big;
    #[cfg(target_endian = "little")]
    /// The target platform’s native endianness.
    pub const NATIVE: Self = Endian::Little;

    /// Converts a byte array containing a UTF-16 [byte order mark] into an
    /// `Endian` value.
    ///
    /// [byte order mark]: https://en.wikipedia.org/wiki/Byte_order_mark
    ///
    /// # Errors
    ///
    /// Returns an error if the input does not contain a byte order mark.
    pub fn from_utf16_bom_bytes(bom: [u8; 2]) -> BinResult<Self> {
        match u16::from_le_bytes(bom) {
            BOM => Ok(Self::Little),
            REVERSE_BOM => Ok(Self::Big),
            _ => Err(crate::Error::BadMagic {
                pos: u64::MAX,
                found: Box::new("Invalid UTF-16 BOM"),
            }),
        }
    }

    /// Converts an `Endian` value into an array containing a UTF-16
    /// [byte order mark](https://en.wikipedia.org/wiki/Byte_order_mark).
    #[must_use]
    pub fn into_utf16_bom_bytes(self) -> [u8; 2] {
        match self {
            Self::Little => u16::to_le_bytes(BOM),
            Self::Big => u16::to_be_bytes(BOM),
        }
    }
}

impl core::fmt::Display for Endian {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Big => write!(f, "Big"),
            Self::Little => write!(f, "Little"),
        }
    }
}

impl Default for Endian {
    fn default() -> Endian {
        Endian::Little
    }
}

const BOM: u16 = 0xFEFF;
const REVERSE_BOM: u16 = 0xFFFE;
