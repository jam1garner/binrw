use binread::BinRead;

#[derive(BinRead)]
enum Foo {
    #[br(magic = 0u8)] A,
    #[br(magic = 1i16)] B,
}

fn main() {}
