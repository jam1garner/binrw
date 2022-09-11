use binrw::io::Cursor;
use binrw::{binwrite, BinWrite};

#[test]
fn pass_args() {
    #[binwrite]
    #[bw(import{ x: u32, y: u8 })]
    struct TestInner {
        #[bw(calc = x)]
        x_copy: u32,

        #[bw(calc = y)]
        y_copy: u8,
    }

    #[derive(BinWrite)]
    #[bw(big)]
    struct Test {
        #[bw(args { x: 1, y: 2 })]
        inner: TestInner,
    }

    let mut x = Cursor::new(Vec::new());
    Test {
        inner: TestInner {},
    }
    .write(&mut x)
    .unwrap();

    assert_eq!(x.into_inner(), b"\0\0\0\x01\x02");
}
