use binrw::BinRead;

#[derive(BinRead)]
struct Foo {
    a: i32,
    #[br(args(0), args_tuple = (a, ))]
    b: i32,
}

fn main() {}
