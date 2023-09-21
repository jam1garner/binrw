use binrw::{io::Cursor, BinRead, BinReaderExt};

#[test]
fn map_args() {
    #[derive(BinRead)]
    #[br(import(offset: u64))]
    #[br(map = |x: u64| Self(x + offset))]
    struct PlusOffset(u64);

    let mut data = Cursor::new([0u8; 8]);

    let PlusOffset(x) = data.read_be_args((20,)).unwrap();

    assert_eq!(x, 20);
}

#[test]
#[should_panic]
fn map_assert() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(assert(false), map(|_: u8| Test {}))]
    struct Test {}

    Test::read(&mut Cursor::new(b"a")).unwrap();
}

#[test]
#[should_panic]
fn map_top_assert_access_fields() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(assert(*x == 2), map(|_: u8| Test { x: 3 }))]
    struct Test {
        x: u8,
    }

    Test::read(&mut Cursor::new(b"a")).unwrap();
}

#[test]
#[should_panic]
fn map_field_assert_access_fields() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(map(|_: u8| Test { x: 3 }))]
    struct Test {
        #[br(assert(*x == 2))]
        x: u8,
    }

    Test::read(&mut Cursor::new(b"a")).expect_err("should fail assertion");
}

#[test]
#[should_panic]
fn map_top_assert_via_self() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(assert(self.x == 2), map(|_: u8| Test { x: 3 }))]
    struct Test {
        x: u8,
    }

    Test::read(&mut Cursor::new(b"a")).unwrap();
}
