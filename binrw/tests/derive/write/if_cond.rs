use binrw::BinWrite;
use binrw::{io::Cursor, Endian, WriteOptions};

#[derive(BinWrite)]
struct Test {
    x: u8,
    #[bw(if(*x > 1, 10))]
    y: u16,
    #[bw(if(*x > 2))]
    z: u32,
}

#[test]
fn if_cond() {
    let mut x = Cursor::new(Vec::new());

    Test { x: 1, y: 2, z: 3 }
        .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    assert_eq!(&x.into_inner(), &[1, 0, 10]);

    let mut x = Cursor::new(Vec::new());

    Test { x: 2, y: 3, z: 4 }
        .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    assert_eq!(&x.into_inner(), &[2, 0, 3]);

    let mut x = Cursor::new(Vec::new());

    Test { x: 3, y: 4, z: 5 }
        .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    assert_eq!(&x.into_inner(), &[3, 0, 4, 0, 0, 0, 5]);
}
