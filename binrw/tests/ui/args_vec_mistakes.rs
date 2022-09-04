use binrw::BinRead;

#[derive(Clone, Copy)]
struct NoDefault;

#[derive(BinRead)]
struct Foo(Vec<u8>);

#[derive(BinRead)]
struct Bar(#[br(args((),))] Vec<u8>);

#[derive(BinRead)]
struct Baz(#[br(args { inner: () })] Vec<u8>);

#[derive(BinRead)]
#[br(import(_a: NoDefault))]
struct Inner {
    _b: u8,
}

#[derive(BinRead)]
struct Qux(#[br(count = 1)] Vec<Inner>);

fn main() {
    Vec::<u8>::read(&mut binrw::io::Cursor::new(b"")).unwrap();
}
