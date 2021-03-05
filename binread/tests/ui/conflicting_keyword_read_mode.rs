use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(calc(1), default, ignore, parse_with = u8)]
    a: i32,
}

fn main() {}
