//! An enum to represent what endianness to read as

use crate::alloc::string::String;

/// An enum to represent what endianness to read as
#[derive(Clone, Copy, Debug)]
pub enum Endian {
    Big,
    Little,
    Native,
}

pub use Endian::{
    Big as BE,
    Little as LE,
    Native as NE
};

impl Endian {
    pub fn from_be_bom(bom: u16) -> Option<Self> {
        match bom {
            0xFEFF => Some(Self::Big),
            0xFFFE => Some(Self::Little),
            _ => None
        }
    }

    pub fn from_le_bom(bom: u16) -> Option<Self> {
        match bom {
            0xFEFF => Some(Self::Little),
            0xFFFE => Some(Self::Big),
            _ => None
        }
    }
}

impl Into<String> for &Endian {
    fn into(self) -> String {
        String::from(
            match self {
                Endian::Big => "Big",
                Endian::Little => "Little",
                Endian::Native => "Native",
            }
        )
    }
}

impl Default for Endian {
    fn default() -> Endian {
        Endian::Native
    }
}
