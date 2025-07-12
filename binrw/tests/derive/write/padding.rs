extern crate binrw;
use super::t;

#[test]
fn padding_big() {
    #[derive(binrw::BinWrite)]
    struct Test(#[bw(pad_size_to = 0x100)] t::Vec<u8>);

    let mut data = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write_le(&Test(t::vec![b'a'; 0x80]), &mut data).unwrap();

    let mut expected = t::vec![0; 0x100];
    expected[0..0x80].fill(b'a');
    t::assert_eq!(data.into_inner(), expected);
}

#[test]
fn padding_round_trip() {
    #[derive(binrw::BinRead, binrw::BinWrite)]
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
    let test = <Test as binrw::BinRead>::read_be(&mut binrw::io::Cursor::new(data)).unwrap();

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(&test, &mut x, binrw::Endian::Big, ()).unwrap();

    t::assert_eq!(x.into_inner(), data);
}

#[test]
fn padding_one_way() {
    #[derive(binrw::BinRead, binrw::BinWrite)]
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

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(
        &Test {
            x: 1,
            y: 2,
            z: 0xabcdef,
        },
        &mut x,
        binrw::Endian::Little,
        (),
    )
    .unwrap();

    t::assert_eq!(x.into_inner(), data);
}
