use binrw::{binrw, io::Cursor, BinRead, BinWrite, Endian, WriteOptions};

#[test]
fn binwrite_temp_applies() {
    #[binrw]
    #[derive(Debug, PartialEq)]
    #[br(big)]
    struct Test {
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
            vec: b"ABCDE".to_vec()
        }
    );
}

#[test]
fn binwrite_temp_with_ignore() {
    #[binrw]
    #[derive(Debug, PartialEq)]
    #[br(big)]
    struct Test {
        #[br(temp)]
        #[bw(ignore)]
        len: u32,

        #[br(count = len)]
        vec: Vec<u8>,
    }

    let result = Test::read(&mut Cursor::new(b"\0\0\0\x05ABCDE")).unwrap();
    assert_eq!(
        result,
        Test {
            vec: b"ABCDE".to_vec()
        }
    );

    let mut x = Cursor::new(Vec::new());

    result
        .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    // Since it's bw(ignore), the length isn't written here
    assert_eq!(x.into_inner(), b"ABCDE");
}
