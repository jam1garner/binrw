extern crate binrw;
use super::t;

#[derive(binrw::BinWrite)]
struct Test {
    x: u8,
    y: u16,
    z: u32,
}

#[test]
fn simple_write() {
    use binrw::BinWrite;

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    Test { x: 1, y: 2, z: 3 }
        .write_options(&mut x, binrw::Endian::Big, ())
        .unwrap();

    t::assert_eq!(x.into_inner(), [1, 0, 2, 0, 0, 0, 3]);
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, PartialEq)]
struct TestRoundTrip {
    x: u16,

    #[brw(little)]
    y: u16,

    #[brw(is_big = true)]
    z: u32,

    #[brw(is_big = false)]
    not_z: u32,
}

#[derive(binrw::BinRead, binrw::BinWrite, Debug, PartialEq)]
struct TestRoundTripConjugate {
    x: u16,

    #[brw(little)]
    y: u16,

    #[brw(is_big = true)]
    z: u32,

    #[brw(is_big = false)]
    not_z: u32,

    empty: (),
}

#[test]
fn round_trip() {
    use binrw::{BinReaderExt, BinWriterExt};

    let bytes = &[0, 1, 2, 0, 0, 0, 0, 3, 3, 0, 0, 0];
    let mut reader = binrw::io::Cursor::new(bytes);
    let test: TestRoundTrip = reader.read_be().unwrap();
    let mut x = binrw::io::Cursor::new(t::Vec::new());
    x.write_be(&test).unwrap();

    t::assert_eq!(x.into_inner(), bytes);
}

#[test]
fn round_trip_2() {
    use binrw::{BinReaderExt, BinWrite};

    let bytes = &[0, 1, 2, 0, 0, 0, 0, 3, 3, 0, 0, 0];
    let mut reader = binrw::io::Cursor::new(bytes);
    let test: TestRoundTrip = reader.read_be().unwrap();

    let bytes_conj = &[0, 1, 2, 0, 0, 0, 0, 3, 3, 0, 0, 0];
    let mut reader_conj = binrw::io::Cursor::new(bytes_conj);
    let conj: TestRoundTripConjugate = reader_conj.read_be().unwrap();

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    let mut y = binrw::io::Cursor::new(t::Vec::new());

    test.write_options(&mut x, binrw::Endian::Big, ()).unwrap();
    conj.write_options(&mut y, binrw::Endian::Big, ()).unwrap();

    t::assert_eq!(x.into_inner() == bytes, y.into_inner() == bytes_conj);
}
