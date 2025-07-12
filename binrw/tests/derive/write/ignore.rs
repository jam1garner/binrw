extern crate binrw;
use super::t;

#[test]
fn ignore_is_not_written() {
    #[binrw::binwrite]
    struct Test {
        #[bw(ignore)]
        x: u32,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(&Test { x: 1 }, &mut x, binrw::Endian::Big, ()).unwrap();

    // Since it's bw(ignore), nothing is written here.
    t::assert_eq!(x.into_inner(), b"");
}
