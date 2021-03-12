use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(if("wrong type"))]
    a: i32,
}

fn main() {}
