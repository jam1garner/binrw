use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    a: i32,
    #[br(args(a), args_tuple = (a, ))]
    b: i32,
}

fn main() {}
