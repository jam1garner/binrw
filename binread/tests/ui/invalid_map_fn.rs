use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(map = does_not_exist)]
    a: i32,
}

fn main() {}
