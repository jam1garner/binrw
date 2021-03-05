use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(parse_with = does_not_exist)]
    a: i32,
}

fn main() {}
