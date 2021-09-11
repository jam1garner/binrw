use binrw::io::Cursor;
use binrw::{BinRead, BinReaderExt, BinWrite, Endian, WriteOptions};

#[test]
fn enum_round_trip() {
    #[derive(BinRead, BinWrite)]
    #[brw(big)]
    enum Test {
        #[brw(magic = b"AAA")]
        A {
            #[brw(little)]
            x: u32,
            y: u8,
        },

        #[brw(little, magic = b"BBB")]
        B {
            x: u32,

            #[brw(big)]
            y: u16,
        },

        #[brw(magic = b"CCC")]
        C,
    }

    let data = b"AAA\x03\x02\x01\0\xFFBBB\xBB\xAA\0\0\0\x02CCC";
    let test: [Test; 3] = Cursor::new(data).read_be().unwrap();

    let mut x = Cursor::new(Vec::new());

    test.write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    assert_eq!(&x.into_inner()[..], data);
}

#[test]
fn enum_one_way() {
    #[derive(BinWrite)]
    #[brw(big)]
    enum Test {
        #[brw(magic = b"AAA")]
        A {
            #[brw(little)]
            x: u32,
            y: u8,
        },

        #[brw(little, magic = b"BBB")]
        B(u32, #[brw(big)] u16),

        #[brw(magic = b"CCC")]
        C,
    }

    let mut x = Cursor::new(Vec::new());

    [
        Test::B(0xAABB, 0x2),
        Test::C,
        Test::A {
            x: 0x10203,
            y: 0xFF,
        },
    ]
    .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
    .unwrap();

    assert_eq!(
        &x.into_inner()[..],
        b"BBB\xBB\xAA\0\0\0\x02CCCAAA\x03\x02\x01\0\xFF"
    );
}
