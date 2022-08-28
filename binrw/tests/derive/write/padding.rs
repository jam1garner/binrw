use binrw::io::Cursor;
use binrw::{BinRead, BinReaderExt, BinWrite, Endian, WriteOptions};

#[test]
fn padding_big() {
    #[derive(BinWrite)]
    struct Test(#[bw(pad_size_to = 0x100)] Vec<u8>);

    let mut data = Cursor::new(Vec::new());
    Test(vec![b'a'; 0x80]).write(&mut data).unwrap();

    let mut expected = vec![0; 0x100];
    expected[0..0x80].fill(b'a');
    assert_eq!(data.into_inner(), expected);
}

#[test]
fn padding_round_trip() {
    #[derive(BinRead, BinWrite)]
    struct Test {
        #[brw(pad_before = 0x2_u32, align_after = 0x8)]
        x: u8,

        #[brw(align_before = 0x4_u32, pad_after = 0x3_u32)]
        y: u8,

        #[brw(pad_size_to = 0x6_u32)]
        z: u32,
    }

    let data = &[
        /* pad_before: */ 0, 0, /* x */ 1, /* align: */ 0, 0, 0, 0, 0,
        /* align_before: (none)*/ /* y */ 2, /* pad_after: */ 0, 0, 0, /* z */ 0,
        0xab, 0xcd, 0xef, /* pad_size_to */ 0, 0,
    ];
    let test: Test = Cursor::new(data).read_be().unwrap();

    let mut x = Cursor::new(Vec::new());

    test.write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    assert_eq!(x.into_inner(), data);
}

#[test]
fn padding_one_way() {
    #[derive(BinRead, BinWrite)]
    struct Test {
        #[brw(pad_before = 0x2_u32, align_after = 0x8)]
        x: u8,

        #[brw(align_before = 0x4_u32, pad_after = 0x3_u32)]
        y: u8,

        #[brw(pad_size_to = 0x6_u32)]
        z: u32,
    }

    let data = &[
        /* pad_before: */ 0, 0, /* x */ 1, /* align: */ 0, 0, 0, 0, 0,
        /* align_before: (none)*/ /* y */ 2, /* pad_after: */ 0, 0, 0, /* z */ 0xef,
        0xcd, 0xab, 0, /* pad_size_to */ 0, 0,
    ];

    let mut x = Cursor::new(Vec::new());

    Test {
        x: 1,
        y: 2,
        z: 0xabcdef,
    }
    .write_options(&mut x, &WriteOptions::new(Endian::Little), ())
    .unwrap();

    assert_eq!(x.into_inner(), data);
}
