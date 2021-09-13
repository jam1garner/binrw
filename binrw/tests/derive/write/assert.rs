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
        dbg!(format!("{}", err));
        assert!(matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        panic!("Assert error expected");
    }
}
