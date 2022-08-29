use binrw::io::Cursor;
use binrw::BinWrite;

#[test]
fn restore_position_writing() {
    #[derive(BinWrite)]
    struct Test {
        #[bw(restore_position)]
        x: u32,
        y: u8,
    }

    let mut x = Vec::new();
    {
        let mut x = Cursor::new(&mut x);
        Test {
            x: 0xffff_ffff,
            y: 0,
        }
        .write_le(&mut x)
        .unwrap();
    }
    assert_eq!(x, b"\0\xff\xff\xff");
}
