use binrw::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(parse_with = 56)]
    a: i32,
}

fn main() {}