extern crate binrw;
use super::t;

#[test]
fn assert_fail() {
    #[binrw::binwrite]
    struct Test {
        #[bw(assert(*x != 1, "x cannot be 1"))]
        x: u32,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    if let t::Err(err) = binrw::BinWrite::write_be(&Test { x: 1 }, &mut x) {
        t::assert!(t::matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        t::panic!("Assert error expected");
    }
}

#[test]
fn top_level_assert_fail() {
    #[binrw::binwrite]
    #[bw(assert(*x != 1, "x cannot be 1"))]
    struct Test {
        x: u32,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    if let t::Err(err) = binrw::BinWrite::write_be(&Test { x: 1 }, &mut x) {
        t::assert!(t::matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        t::panic!("Assert error expected");
    }
}

#[test]
fn top_level_assert_self_enum() {
    #[binrw::binwrite]
    #[bw(assert(!t::matches!(self, Test::A(1))))]
    #[derive(PartialEq)]
    enum Test {
        A(u32),
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    if let t::Err(err) = binrw::BinWrite::write_be(&Test::A(1), &mut x) {
        t::assert!(t::matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        t::panic!("Assert error expected");
    }
}

#[test]
fn assert_enum_variant() {
    #[binrw::binwrite]
    #[derive(PartialEq)]
    enum Test {
        #[bw(assert(self_0 != &1))]
        A(u32),
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    if let t::Err(err) = binrw::BinWrite::write_be(&Test::A(1), &mut x) {
        t::assert!(t::matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        t::panic!("Assert error expected");
    }
}

#[test]
fn top_level_assert_self_struct() {
    #[binrw::binwrite]
    #[bw(assert(self != &Test(1)))]
    #[derive(PartialEq)]
    struct Test(u32);

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    if let t::Err(err) = binrw::BinWrite::write_be(&Test(1), &mut x) {
        t::assert!(t::matches!(err, binrw::Error::AssertFail { .. }));
    } else {
        t::panic!("Assert error expected");
    }
}
