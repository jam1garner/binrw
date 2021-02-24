use binread::BinRead;

#[derive(BinRead)]
#[br(assert(false, "message", "too", "many", "arguments"))]
struct Foo;

fn main() {}
