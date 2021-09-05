use binrw::binwrite;
use binrw::BinWrite;
use binrw::{io::Cursor, Endian, WriteOptions};

#[binwrite]
struct Test {
    x: u8,
    y: u16,
    z: u32,
}

#[test]
fn simple_write() {
    let mut x = Cursor::new(Vec::new());

    Test { x: 1, y: 2, z: 3 }
        .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    assert_eq!(&x.into_inner()[..], &[1, 0, 2, 0, 0, 0, 3]);
}

use binrw::binread;
use binrw::BinReaderExt;

#[binread]
#[binwrite]
#[derive(Debug)]
struct TestRoundTrip {
    x: u16,

    #[brw(little)]
    y: u16,

    #[brw(is_big = true)]
    z: u32,

    #[brw(is_big = false)]
    not_z: u32,
}

#[test]
fn round_trip() {
    let bytes = &[0, 1, 2, 0, 0, 0, 0, 3, 3, 0, 0, 0];

    let mut reader = Cursor::new(bytes);

    let test: TestRoundTrip = reader.read_be().unwrap();

    let mut x = Cursor::new(Vec::new());

    test.write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    assert_eq!(&x.into_inner()[..], bytes);
}
