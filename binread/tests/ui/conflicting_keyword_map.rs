use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(map = |_| 0, try_map = |_| Ok(0))]
    a: i32,
}

fn main() {}
