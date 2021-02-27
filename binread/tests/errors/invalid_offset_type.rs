use binread::{BinRead, FilePtr};

#[derive(BinRead)]
struct Test {
    a: u8,
    #[br(offset = a)]
    b: FilePtr<u8, u8>,
    #[br(offset_after = d)]
    c: FilePtr<u8, u8>,
    d: u8,
}

fn main() {}
