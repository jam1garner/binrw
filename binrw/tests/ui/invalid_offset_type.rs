use binrw::{BinRead, FilePtr};

#[derive(BinRead)]
struct Test {
    a: u8,
    #[br(offset = a)]
    b: FilePtr<u8, u8>,
}

fn main() {}
