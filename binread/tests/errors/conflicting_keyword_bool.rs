use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(ignore, ignore)]
    a: i32,
}

fn main() {}
