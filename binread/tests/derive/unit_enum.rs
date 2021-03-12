use binread::{BinRead, io::{Cursor, Seek, SeekFrom}};

#[test]
fn unit_enum_magic() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        // First variant not having any magic ensures that there is no reliance
        // internally on a specific variant having a magic
        #[allow(dead_code)]
        Zero,
        #[br(magic(1u16))]
        One,
        #[br(magic(2u16))]
        Two,
    }

    let error = Test::read(&mut Cursor::new(b"\0\0")).expect_err("accepted bad data");
    assert!(matches!(error, binread::Error::NoVariantMatch { .. }));
    assert_eq!(Test::read(&mut Cursor::new(b"\0\x01")).unwrap(), Test::One);
    assert_eq!(Test::read(&mut Cursor::new(b"\0\x02")).unwrap(), Test::Two);
}

#[test]
fn unit_enum_magic_bytes() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(b"zero"))]
        Zero,
        #[allow(dead_code)]
        One,
        #[br(magic(b"two0"))]
        Two,
    }

    assert_eq!(Test::read(&mut Cursor::new(b"zero")).unwrap(), Test::Zero);
    let error = Test::read(&mut Cursor::new(b"oops")).expect_err("accepted bad data");
    assert!(matches!(error, binread::Error::NoVariantMatch { .. }));
    assert_eq!(Test::read(&mut Cursor::new(b"two0")).unwrap(), Test::Two);
}

#[test]
fn unit_enum_magic_pre_assert() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(big, import(allow_zero: bool, forbid_zero: bool))]
    enum Test {
        #[br(magic(0u16), pre_assert(allow_zero))]
        // This redundant check is intentional and tests that assertions are
        // combining properly
        #[br(pre_assert(!forbid_zero))]
        Zero,
        #[br(magic(0u16))]
        OtherZero,
    }

    assert_eq!(Test::read_args(&mut Cursor::new(b"\0\0"), (true, false)).unwrap(), Test::Zero);
    // Tests allow_zero condition actually applies
    assert_eq!(Test::read_args(&mut Cursor::new(b"\0\0"), (true, true)).unwrap(), Test::OtherZero);
    // Tests forbid_zero condition actually applies
    assert_eq!(Test::read_args(&mut Cursor::new(b"\0\0"), (false, true)).unwrap(), Test::OtherZero);
    let error = Test::read_args(&mut Cursor::new(b"\0\x01"), (false, true)).expect_err("accepted bad data");
    assert!(matches!(error, binread::Error::NoVariantMatch { .. }));
}

#[test]
fn unit_enum_repr() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(big, repr(i16))]
    enum Test {
        Neg1 = -1,
        Zero,
        Two = 2,
    }

    assert_eq!(Test::read(&mut Cursor::new(b"\xff\xff")).unwrap(), Test::Neg1);
    let error = Test::read(&mut Cursor::new(b"\0\x01")).expect_err("accepted bad data");
    assert!(matches!(error, binread::Error::NoVariantMatch { .. }));
    assert_eq!(Test::read(&mut Cursor::new(b"\0\x02")).unwrap(), Test::Two);
}

#[test]
fn unit_enum_rewind_on_eof() {
    #[derive(BinRead, Debug)]
    #[br(repr(u16))]
    enum Test {
        A,
    }

    let mut data = Cursor::new(b"\0\0");
    let expected = data.seek(SeekFrom::Start(1)).unwrap();
    Test::read(&mut data).expect_err("accepted bad data");
    assert_eq!(expected, data.seek(SeekFrom::Current(0)).unwrap());
}

#[test]
fn unit_enum_rewind_on_no_variant() {
    #[derive(BinRead, Debug)]
    #[br(repr(u8))]
    enum Test {
        A = 1,
    }

    let mut data = Cursor::new(b"\0\0");
    let expected = data.seek(SeekFrom::Start(1)).unwrap();
    Test::read(&mut data).expect_err("accepted bad data");
    assert_eq!(expected, data.seek(SeekFrom::Current(0)).unwrap());
}
