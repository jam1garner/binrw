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
