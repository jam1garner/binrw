use binrw::BinRead;

#[derive(Clone, Copy)]
struct NoDefault;

#[derive(BinRead)]
struct MissingArgs(Vec<u8>);

#[derive(BinRead)]
struct WrongType(#[br(args((),))] Vec<u8>);

#[derive(BinRead)]
struct MissingCount(#[br(args { inner: () })] Vec<u8>);

#[derive(BinRead)]
#[br(import(_a: NoDefault))]
struct Inner {
    _b: u8,
}

#[derive(BinRead)]
struct WrongCountType(#[br(count = Some(1))] Vec<u8>);

#[derive(BinRead)]
struct MissingInnerArgs(#[br(count = 1)] Vec<Inner>);

fn main() {
    Vec::<u8>::read(&mut binrw::io::Cursor::new(b"")).unwrap();
}
