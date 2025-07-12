extern crate binrw;
use super::t;

#[test]
fn restore_position_writing() {
    #[derive(binrw::BinWrite)]
    struct Test {
        #[bw(restore_position)]
        x: u32,
        y: u8,
    }

    let mut x = t::Vec::new();
    {
        let mut x = binrw::io::Cursor::new(&mut x);
        binrw::BinWrite::write_le(
            &Test {
                x: 0xffff_ffff,
                y: 0,
            },
            &mut x,
        )
        .unwrap();
    }
    t::assert_eq!(x, b"\0\xff\xff\xff");
}
