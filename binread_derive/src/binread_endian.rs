/// An enum to represent the endianess to read
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Endian {
    Big,
    Little,
    Native,
}

impl Default for Endian {
    fn default() -> Self {
        Self::Native
    }
}
