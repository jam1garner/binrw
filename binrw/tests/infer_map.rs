use binrw::{binread, BinReaderExt};

#[binread]
#[derive(Debug, Eq, PartialEq)]
pub struct LenString {
    #[br(temp)]
    name_len: u8,
    #[br(count = name_len, map = |bytes: Vec<u8>| String::from_utf8_lossy(&bytes).into_owned())]
    name: String,
}

#[test]
fn parse_len_string() {
    let mut data = binrw::io::Cursor::new(b"\x03cat");

    let result: LenString = data.read_le().unwrap();

    assert_eq!(
        result,
        LenString {
            name: String::from("cat")
        }
    );
}
