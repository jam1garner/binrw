extern crate binrw;
use super::t;

#[test]
fn pass_args() {
    #[binrw::binwrite]
    #[bw(import{ x: u32, y: u8 })]
    struct TestInner {
        #[bw(calc = x)]
        x_copy: u32,

        #[bw(calc = y)]
        y_copy: u8,
    }

    #[derive(binrw::BinWrite)]
    #[bw(big)]
    struct Test {
        #[bw(args { x: 1, y: 2 })]
        inner: TestInner,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write(
        &Test {
            inner: TestInner {},
        },
        &mut x,
    )
    .unwrap();

    t::assert_eq!(x.into_inner(), b"\0\0\0\x01\x02");
}
