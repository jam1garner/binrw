use binrw::binwrite;
use binrw::BinWrite;
use binrw::{WriteOptions, io::Cursor, Endian};

#[binwrite]
struct Test {
    x: u8,
    y: u16,
    z: u32,
}

#[test]
fn simple_write() {
    let mut x = Cursor::new(Vec::new());

    Test {
        x: 1,
        y: 2,
        z: 3,
    }.write_options(&mut x, &WriteOptions::new(Endian::Big), ()).unwrap();

    assert_eq!(&x.into_inner()[..], &[1, 0, 2, 0, 0, 0, 3]);
}

#[binwrite]
struct TestEndian {
    x: u16,

    #[bw(little)]
    y: u16,

    #[bw(is_big = true)]
    z: u32,

    #[bw(is_big = false)]
    not_z: u32,
}

#[test]
fn write_endian() {
    let mut x = Cursor::new(Vec::new());

    TestEndian {
        x: 1,
        y: 2,
        z: 3,
        not_z: 3,
    }.write_options(&mut x, &WriteOptions::new(Endian::Big), ()).unwrap();

    assert_eq!(&x.into_inner()[..], &[0, 1, 2, 0, 0, 0, 0, 3, 3, 0, 0, 0]);
}
