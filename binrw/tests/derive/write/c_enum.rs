use binrw::io::Cursor;
use binrw::{BinRead, BinReaderExt, BinWrite, Endian};

#[test]
fn write_enum() {
    #[derive(BinWrite)]
    #[bw(repr(u32))]
    enum Test {
        A,
        B = 3,
        C,
        D = 5,
    }

    let mut x = Cursor::new(Vec::new());

    vec![Test::A, Test::B, Test::C, Test::D]
        .write_options(&mut x, Endian::Big, ())
        .unwrap();

    assert_eq!(
        x.into_inner(),
        [0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5]
    );
}

#[test]
fn round_trip_unit_enum() {
    #[derive(BinRead, BinWrite)]
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
    let test: [Test; 4] = Cursor::new(data).read_be().unwrap();

    let mut x = Cursor::new(Vec::new());

    test.write_options(&mut x, Endian::Big, ()).unwrap();

    assert_eq!(x.into_inner(), data);
}

#[test]
fn magic_enum_round_trip() {
    #[derive(BinRead, BinWrite)]
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
    let test: [Test; 4] = Cursor::new(data).read_be().unwrap();

    let mut x = Cursor::new(Vec::new());

    test.write_options(&mut x, Endian::Big, ()).unwrap();

    assert_eq!(x.into_inner(), data);
}
