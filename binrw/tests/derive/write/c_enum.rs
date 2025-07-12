extern crate binrw;
use super::t;

#[test]
fn write_enum() {
    #[derive(binrw::BinWrite)]
    #[bw(repr(u32))]
    enum Test {
        A,
        B = 3,
        C,
        D = 5,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(
        &t::vec![Test::A, Test::B, Test::C, Test::D],
        &mut x,
        binrw::Endian::Big,
        (),
    )
    .unwrap();

    t::assert_eq!(
        x.into_inner(),
        [0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5]
    );
}

#[test]
fn round_trip_unit_enum() {
    #[derive(binrw::BinRead, binrw::BinWrite)]
    #[brw(repr(u32), magic = 0xff_u8)]
    enum Test {
        A,
        B = 3,
        C,
        D = 5,
    }

    let data = &[
        0xff, 0, 0, 0, 0, 0xff, 0, 0, 0, 3, 0xff, 0, 0, 0, 4, 0xff, 0, 0, 0, 5,
    ];
    let test: [Test; 4] = binrw::BinReaderExt::read_be(&mut binrw::io::Cursor::new(data)).unwrap();

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(&test, &mut x, binrw::Endian::Big, ()).unwrap();

    t::assert_eq!(x.into_inner(), data);
}

#[test]
fn magic_enum_round_trip() {
    #[derive(binrw::BinRead, binrw::BinWrite)]
    enum Test {
        #[brw(magic = b"abc")]
        A,

        #[brw(magic = b"123")]
        B,

        #[brw(magic = b"def")]
        C,

        #[brw(magic = b"456")]
        D,
    }

    let data = b"123abcdef456";
    let test: [Test; 4] = binrw::BinReaderExt::read_be(&mut binrw::io::Cursor::new(data)).unwrap();

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(&test, &mut x, binrw::Endian::Big, ()).unwrap();

    t::assert_eq!(x.into_inner(), data);
}
