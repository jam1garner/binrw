use binrw::BinRead;

#[derive(BinRead)]
#[br(magic = 0)]
struct Foo;

fn main() {}
