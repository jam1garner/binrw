use binrw::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(try, calc(None))]
    a: Option<u8>,
}

fn main() {}
