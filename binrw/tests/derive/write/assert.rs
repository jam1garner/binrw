use binrw::{binwrite, io::Cursor, BinWriterExt};

#[test]
fn assert_fail() {
    #[binwrite]
    struct Test {
        #[bw(assert(*x != 1, "x cannot be 1"))]
        x: u32,
    }

    let mut x = Cursor::new(Vec::new());
    if let Err(err) = x.write_be(&Test { x: 1 }) {
        assert!(matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        panic!("Assert error expected");
    }
}

#[test]
fn top_level_assert_fail() {
    #[binwrite]
    #[bw(assert(*x != 1, "x cannot be 1"))]
    struct Test {
        x: u32,
    }

    let mut x = Cursor::new(Vec::new());
    if let Err(err) = x.write_be(&Test { x: 1 }) {
        assert!(matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        panic!("Assert error expected");
    }
}
