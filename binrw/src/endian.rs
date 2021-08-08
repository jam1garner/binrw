//! Type definitions for byte order handling.

use crate::BinResult;

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

const BOM: u16 = 0xFEFF;
const REVERSE_BOM: u16 = 0xFFFE;

impl Endian {
    /// Converts from a UTF-16 BOM (either `[0xFF, 0xFE]` or `[0xFE, 0xFF]`) into the endian it
    /// represents
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

    /// Converts endian to a UTF-16 BOM representing the given endian
    pub fn into_utf16_bom_bytes(&self) -> [u8; 2] {
        match self {
            Self::Little => u16::to_le_bytes(BOM),
            Self::Big => u16::to_be_bytes(BOM),
            Self::Native => u16::to_ne_bytes(BOM),
        }
    }
}
