use binrw::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(try, default)]
    a: Option<u8>,
}

fn main() {}
