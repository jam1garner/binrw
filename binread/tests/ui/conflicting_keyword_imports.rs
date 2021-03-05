use binread::BinRead;

#[derive(BinRead)]
#[br(import(a: i32), import_tuple(args: (i32, )))]
struct Foo;

fn main() {}
