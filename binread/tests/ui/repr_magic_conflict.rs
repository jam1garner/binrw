use binread::BinRead;

#[derive(BinRead)]
#[br(repr = u8)]
enum Foo {
    #[br(magic = 0u8)] A,
}

fn main() {}
