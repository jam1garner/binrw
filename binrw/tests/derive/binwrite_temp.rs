use binrw::{binrw, io::Cursor, BinRead};

#[test]
fn binwrite_temp_applies() {
    #[binrw]
    #[derive(Debug, PartialEq)]
    #[br(big)]
    struct Test {
        #[br(temp)]
        #[bw(calc = vec.len() as u32)]
        len: u32,

        #[br(count = len)]
        vec: Vec<u8>,
    }

    let result = Test::read(&mut Cursor::new(b"\0\0\0\x05ABCDE")).unwrap();
    // This also indirectly checks that `temp` is actually working since
    // compilation would fail if it weren’t due to missing the `len` field
    assert_eq!(
        result,
        Test {
            vec: Vec::from(&b"ABCDE"[..])
        }
    );
}
