use binrw::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(err_context("too", "many", "arguments"))]
    a: u8,
}

#[derive(BinRead)]
struct Bar {
    #[br(err_context())]
    b: u8,
}

#[derive(BinRead)]
struct Baz {
    #[br(err_context(a, b))]
    c: u8,
}

fn main() {}
