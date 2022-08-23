use binrw::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(args(()), calc(None))]
    a: Option<u8>,
}

fn main() {}
