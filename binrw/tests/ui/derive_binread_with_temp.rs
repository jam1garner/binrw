use binrw::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(temp)]
    a: u8,
}

fn main() {}
