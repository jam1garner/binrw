#[allow(unused_imports)]
use super::*;

/// An enum to represent what endianness to write with
#[derive(Clone, Copy, Debug)]
pub enum Endian {
    Big,
    Little,
    Native,
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
