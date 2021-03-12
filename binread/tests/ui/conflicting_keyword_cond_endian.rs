use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(big, little, is_big = true, is_little = true)]
    a: i32,
}

fn main() {}
