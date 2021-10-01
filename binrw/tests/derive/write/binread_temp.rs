use binrw::{binrw, io::Cursor, BinWrite, Endian, WriteOptions};

#[test]
fn binread_temp_applies() {
    #[binrw]
    #[bw(import { x: u32})]
    struct TestInner {
        #[br(ignore)]
        #[bw(calc = x)]
        x_copy: u32,
    }

    #[binrw]
    #[bw(big)]
    struct Test {
        #[bw(args { x: 1 })]
        inner: TestInner,
    }

    let mut x = Cursor::new(Vec::new());

    Test {
        inner: TestInner {},
    }
    .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
    .unwrap();

    assert_eq!(&x.into_inner()[..], &[0, 0, 0, 1]);
}
