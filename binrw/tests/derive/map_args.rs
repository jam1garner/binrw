extern crate binrw;
use super::t;

#[test]
fn map_args() {
    #[derive(binrw::BinRead)]
    #[br(import(offset: u64))]
    #[br(map = |x: u64| Self(x + offset))]
    struct PlusOffset(u64);

    let mut data = binrw::io::Cursor::new([0u8; 8]);
    let PlusOffset(x) = binrw::BinReaderExt::read_be_args(&mut data, (20,)).unwrap();
    t::assert_eq!(x, 20);
}

#[test]
fn map_assert() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(assert(false), map(|_: u8| Test {}))]
    struct Test {}

    <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"a"))
        .expect_err("should fail assertion");
}

#[test]
fn map_top_assert_access_fields() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(assert(*x == 2), map(|_: u8| Test { x: 3 }))]
    struct Test {
        x: u8,
    }

    <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"a"))
        .expect_err("should fail assertion");
}

#[test]
fn map_field_assert_access_fields() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(map(|_: u8| Test { x: 3 }))]
    struct Test {
        #[br(assert(*x == 2))]
        x: u8,
    }

    <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"a"))
        .expect_err("should fail assertion");
}

#[test]
fn map_top_assert_via_self() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(assert(self.x == 2), map(|_: u8| Test { x: 3 }))]
    struct Test {
        x: u8,
    }

    <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"a"))
        .expect_err("should fail assertion");
}
