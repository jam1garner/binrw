use binread::BinRead;

#[derive(BinRead)]
#[br(magic = 0u8, magic = 0u8)]
struct Foo;

fn main() {}
