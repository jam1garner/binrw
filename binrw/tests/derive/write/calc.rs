use binrw::{binwrite, io::Cursor, BinWrite, Endian, WriteOptions};

#[test]
fn calc_simple_write() {
    #[binwrite]
    struct Test {
        x: u8,

        #[bw(calc = 2)]
        y: u16,

        #[bw(calc = (*x as u32) + 2)]
        z: u32,
    }

    let mut x = Cursor::new(Vec::new());

    Test { x: 1 }
        .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    assert_eq!(&x.into_inner()[..], &[1, 0, 2, 0, 0, 0, 3]);
}
