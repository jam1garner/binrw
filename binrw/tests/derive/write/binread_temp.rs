extern crate binrw;
use super::t;

#[test]
fn binread_temp_applies() {
    #[binrw::binrw]
    #[bw(import { x: u32 })]
    struct TestInner {
        #[br(ignore)]
        #[bw(calc = x)]
        x_copy: u32,
    }

    #[binrw::binrw]
    #[bw(big)]
    struct Test {
        #[bw(args { x: 1 })]
        inner: TestInner,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(
        &Test {
            inner: TestInner {},
        },
        &mut x,
        binrw::Endian::Big,
        (),
    )
    .unwrap();

    t::assert_eq!(x.into_inner(), [0, 0, 0, 1]);
}
