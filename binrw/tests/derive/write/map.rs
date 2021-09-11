use binrw::io::Cursor;
use binrw::{binwrite, BinWrite};

#[test]
fn map_field() {
    #[binwrite]
    #[bw(big)]
    struct Test {
        #[bw(map = |&x| x as u64)]
        x: u32,

        #[bw(map = |x| x.as_bytes())]
        y: String,

        #[bw(calc = 0xff, map = |x: u8| x)]
        z: u8,
    }

    let mut x = Cursor::new(Vec::new());

    Test {
        x: 3,
        y: String::from("test"),
    }.write_to(&mut x).unwrap();

    assert_eq!(&x.into_inner()[..], b"\0\0\0\0\0\0\0\x03test\xff");
}

#[test]
fn map_field_code_coverage() {
    #[derive(BinWrite)]
    struct Test {
        #[bw(map = |&x| x as u64)]
        x: u32,

        #[bw(map = |x| x.as_bytes())]
        y: String,
    }
}
