/// An enum to represent the endianess to read
#[derive(Clone, Copy, Debug, PartialEq)]
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
