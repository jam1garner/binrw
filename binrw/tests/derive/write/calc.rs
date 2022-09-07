use binrw::{binwrite, io::Cursor, BinWrite, Endian};

#[test]
fn calc_simple_write() {
    #[binwrite]
    struct Test {
        x: u8,

        #[bw(calc = Some(2))]
        y: Option<u16>,

        #[bw(calc = (*x as u32) + 2)]
        z: u32,
    }

    let mut x = Cursor::new(Vec::new());

    Test { x: 1 }
        .write_options(&mut x, Endian::Big, ())
        .unwrap();

    assert_eq!(x.into_inner(), [1, 0, 2, 0, 0, 0, 3]);
}

#[test]
fn calc_visibility() {
    #[binwrite]
    struct Test {
        x: u8,

        #[bw(calc = Some(2))]
        y: Option<u16>,

        // `y` should be visible here even though it is calculated
        #[bw(calc = y.unwrap() + 1)]
        z: u16,
    }

    let mut x = Cursor::new(Vec::new());

    Test { x: 1 }
        .write_options(&mut x, Endian::Big, ())
        .unwrap();

    assert_eq!(x.into_inner(), [1, 0, 2, 0, 3]);
}
