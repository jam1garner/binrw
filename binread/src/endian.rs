//! An enum to represent what endianness to read as

/// An enum to represent what endianness to read as
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
