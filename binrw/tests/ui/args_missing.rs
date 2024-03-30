use binrw::BinRead;

#[derive(Clone, Copy)]
struct NoDefault;

#[derive(BinRead)]
#[br(import(_a: NoDefault))]
struct Foo;

#[derive(BinRead)]
struct Bar {
    a: Foo,
}

fn main() {}
