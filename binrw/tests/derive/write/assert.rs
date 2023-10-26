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

#[test]
fn top_level_assert_self_enum() {
    #[binwrite]
    #[bw(assert(!matches!(self, Test::A(1))))]
    #[derive(PartialEq)]
    enum Test {
        A(u32),
    }

    let mut x = Cursor::new(Vec::new());
    if let Err(err) = x.write_be(&Test::A(1)) {
        assert!(matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        panic!("Assert error expected");
    }
}

#[test]
fn assert_enum_variant() {
    #[binwrite]
    #[derive(PartialEq)]
    enum Test {
        #[bw(assert(self_0 != &1))]
        A(u32),
    }

    let mut x = Cursor::new(Vec::new());
    if let Err(err) = x.write_be(&Test::A(1)) {
        assert!(matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        panic!("Assert error expected");
    }
}

#[test]
fn top_level_assert_self_struct() {
    #[binwrite]
    #[bw(assert(self != &Test(1)))]
    #[derive(PartialEq)]
    struct Test(u32);

    let mut x = Cursor::new(Vec::new());
    if let Err(err) = x.write_be(&Test(1)) {
        assert!(matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        panic!("Assert error expected");
    }
}
