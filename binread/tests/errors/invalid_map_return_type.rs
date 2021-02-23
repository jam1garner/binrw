use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(map = |_| 0u8)]
    a: i32,
}

fn main() {}
