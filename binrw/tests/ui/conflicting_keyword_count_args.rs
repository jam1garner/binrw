use binrw::BinRead;

#[derive(BinRead)]
#[br(import(a: u8))]
struct Item;

#[derive(BinRead)]
struct Foo {
    a: u8,
    b: u8,
    #[br(count = b, args(a))]
    c: Vec<Item>,
}

#[derive(BinRead)]
struct Bar {
    a: u8,
    b: u8,
    #[br(count = b, args_raw = a)]
    c: Vec<Item>,
}

#[derive(BinRead)]
struct Baz {
    a: u8,
    b: u8,
    #[br(count = b, args(a, b))]
    c: Vec<Item>,
}

fn main() {}
