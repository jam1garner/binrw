use binrw::{io::Cursor, BinRead, BinReaderExt, BinWrite};

#[test]
fn magic_round_trip() {
    #[derive(BinRead, BinWrite)]
    #[brw(little, magic = 0x1234_5678_u32)]
    struct Test {
        #[brw(big, magic = 0x9ABC_u16)]
        x: u32,

        y: u16,
    }

    let data = [0x78, 0x56, 0x34, 0x12, 0x9A, 0xBC, 0, 0, 0, 1, 0x3, 0];

    let test: Test = Cursor::new(data).read_be().unwrap();

    let mut out = Cursor::new(Vec::new());
    test.write_args(&mut out, ()).unwrap();

    assert_eq!(out.into_inner(), data);
}

#[test]
fn magic_one_way() {
    #[derive(BinRead, BinWrite)]
    #[bw(little, magic = b"ABCD")]
    struct Test {
        #[bw(big, magic = 0x9ABC_u16)]
        x: u32,

        y: u16,
    }

    let mut out = Cursor::new(Vec::new());
    Test { x: 1, y: 5 }.write(&mut out).unwrap();

    let data = b"ABCD\x9A\xBC\0\0\0\x01\x05\0";

    assert_eq!(out.into_inner(), data);
}
