extern crate binrw;
use super::t;

#[test]
fn if_cond() {
    #[derive(binrw::BinWrite)]
    struct Test {
        x: u8,
        #[bw(if(*x > 1, 10))]
        y: u16,
        #[bw(if(*x > 2))]
        z: u32,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write_options(&Test { x: 1, y: 2, z: 3 }, &mut x, binrw::Endian::Big, ())
        .unwrap();
    t::assert_eq!(&x.into_inner(), &[1, 0, 10]);

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write_options(&Test { x: 2, y: 3, z: 4 }, &mut x, binrw::Endian::Big, ())
        .unwrap();
    t::assert_eq!(&x.into_inner(), &[2, 0, 3]);

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write_options(&Test { x: 3, y: 4, z: 5 }, &mut x, binrw::Endian::Big, ())
        .unwrap();
    t::assert_eq!(&x.into_inner(), &[3, 0, 4, 0, 0, 0, 5]);
}
