use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(if(false, 0, 1, 2, 3))]
    a: u8,
}

fn main() {}
