use binrw::binrw;
use binrw::BinWrite;

#[test]
fn correct_args_type_set() {
    #[derive(BinWrite)]
    #[bw(import { _x: u32, _y: u8 })]
    struct Test {}

    let mut x = binrw::io::Cursor::new(Vec::new());

    Test {}
        .write_options(&mut x, &Default::default(), binrw::args! { _x: 3, _y: 2 })
        .unwrap();
}

#[test]
fn usable_args() {
    #[binrw]
    #[bw(import { x: u32, _y: u8 })]
    struct Test {
        #[br(temp, ignore)]
        #[bw(calc = x)]
        x_copy: u32,
    }

    let mut x = binrw::io::Cursor::new(Vec::new());

    Test {}
        .write_options(&mut x, &Default::default(), binrw::args! { x: 3, _y: 2 })
        .unwrap();
}
