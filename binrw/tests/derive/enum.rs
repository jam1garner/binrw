use binrw::{
    binread,
    io::{Cursor, Seek, SeekFrom},
    BinRead,
};

#[test]
fn enum_assert() {
    #[derive(BinRead, Debug, PartialEq)]
    #[br(assert(b == 1))]
    enum Test {
        A { a: u8, b: u8 },
        B { a: i16, b: u8 },
    }

    let mut data = Cursor::new(b"\xff\xff\x01");
    let result = Test::read(&mut data).unwrap();
    assert_eq!(result, Test::B { a: -1, b: 1 });
}

#[test]
fn enum_non_copy_args() {
    #[derive(BinRead, Debug)]
    #[br(import(a: NonCopyArg))]
    enum Test {
        A {
            #[br(calc = a.0)]
            a: u8,
        },
        B {
            #[br(calc = a.0)]
            b: u8,
        },
    }

    #[derive(Clone)]
    struct NonCopyArg(u8);
}

#[test]
fn enum_calc_temp_field() {
    #[binread]
    #[derive(Debug, Eq, PartialEq)]
    enum Test {
        #[br(magic(0u8))]
        Zero {
            #[br(temp)]
            a: u8,
            #[br(calc(a))]
            b: u8,
        },
    }

    let result = Test::read(&mut Cursor::new(b"\0\x04")).unwrap();
    // This also indirectly checks that `temp` is actually working since
    // compilation would fail if it weren’t due to the missing `a` property
    assert_eq!(result, Test::Zero { b: 4 });
}

#[test]
fn enum_endianness() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(1u16))]
        OneBig,
        #[br(little, magic(2u16))]
        TwoLittle { a: u16 },
    }

    assert_eq!(
        Test::read(&mut Cursor::new(b"\0\x01")).unwrap(),
        Test::OneBig
    );
    let error = Test::read(&mut Cursor::new(b"\x01\0")).expect_err("accepted bad data");
    assert!(matches!(error, binrw::Error::EnumErrors { .. }));
    assert_eq!(
        Test::read(&mut Cursor::new(b"\x02\0\x03\0")).unwrap(),
        Test::TwoLittle { a: 3 }
    );
    let error = Test::read(&mut Cursor::new(b"\0\x02\x03\0")).expect_err("accepted bad data");
    assert!(matches!(error, binrw::Error::EnumErrors { .. }));
}

#[test]
fn enum_magic() {
    #[derive(BinRead, Debug, PartialEq)]
    #[br(big, magic(0x1234u16))]
    enum Test {
        #[br(magic(0u8))]
        Zero { a: u16 },
        // Fail on the second field to actually test that a rewind happens
        // to the beginning of the enum data, not to before the enum magic
        #[br(magic(1u8))]
        One { a: u16 },
    }

    let result = Test::read(&mut Cursor::new(b"\x12\x34\x01\x02\x03")).unwrap();
    assert_eq!(result, Test::One { a: 515 });
}

#[test]
fn enum_return_all_errors() {
    #[derive(BinRead, Debug)]
    #[br(big, return_all_errors)]
    enum Test {
        #[br(magic(0u16))]
        One { a: u16 },
        #[br(magic(1u16))]
        Two { a: u16 },
    }

    let error = Test::read(&mut Cursor::new("\0\x01")).expect_err("accepted bad data");
    match error {
        binrw::Error::EnumErrors {
            pos,
            variant_errors,
        } => {
            assert_eq!(pos, 0);
            assert_eq!(variant_errors.len(), 2);
            assert_eq!(variant_errors[0].0, "One");
            if let binrw::Error::BadMagic { pos, found } = &variant_errors[0].1 {
                assert_eq!(pos, &0);
                assert_eq!(&format!("{:?}", found), "1");
            } else {
                panic!("expected BadMagic; got {:?}", variant_errors[0].1);
            }
            assert_eq!(variant_errors[1].0, "Two");
            assert!(matches!(variant_errors[1].1, binrw::Error::Io(..)));
        }
        _ => panic!("wrong error type"),
    }
}

#[test]
fn enum_rewind_on_assert() {
    #[derive(BinRead, Debug)]
    #[br(assert(b == 1))]
    enum Test {
        A { a: u8, b: u8 },
        B { a: u16, b: u8 },
    }

    let mut data = Cursor::new(b"\0\0\0\0");
    let expected = data.seek(SeekFrom::Start(1)).unwrap();
    Test::read(&mut data).expect_err("accepted bad data");
    assert_eq!(expected, data.seek(SeekFrom::Current(0)).unwrap());
}

#[test]
fn enum_rewind_on_eof() {
    #[derive(BinRead, Debug)]
    enum Test {
        A {
            a: u8,
            // Fail on the second field to actually test that a rewind happens
            // to the beginning of the enum, not just the beginning of the field
            b: u16,
        },
    }

    let mut data = Cursor::new(b"\0\0\0");
    let expected = data.seek(SeekFrom::Start(1)).unwrap();
    Test::read(&mut data).expect_err("accepted bad data");
    assert_eq!(expected, data.seek(SeekFrom::Current(0)).unwrap());
}

#[test]
fn enum_rewind_on_variant_assert() {
    #[derive(BinRead, Debug)]
    enum Test {
        #[br(assert(b == 1))]
        A { a: u8, b: u8 },
    }

    let mut data = Cursor::new(b"\0\0");
    let expected = data.seek(SeekFrom::Start(1)).unwrap();
    Test::read(&mut data).expect_err("accepted bad data");
    assert_eq!(expected, data.seek(SeekFrom::Current(0)).unwrap());
}

#[test]
fn enum_return_unexpected_error() {
    #[derive(BinRead, Debug)]
    #[br(big, return_unexpected_error)]
    enum Test {
        #[br(magic(0u16))]
        One { a: u16 },
        #[br(magic(1u16))]
        Two { a: u16 },
    }

    let error = Test::read(&mut Cursor::new("\0\x01")).expect_err("accepted bad data");
    assert!(matches!(error, binrw::Error::NoVariantMatch { .. }));
}

#[test]
fn mixed_enum() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(0u8))]
        Zero,
        #[br(magic(2u8))]
        Two { a: u16, b: u16 },
    }

    assert!(matches!(
        Test::read(&mut Cursor::new(b"\0")).unwrap(),
        Test::Zero
    ));
    let error = Test::read(&mut Cursor::new(b"\x01")).expect_err("accepted bad data");
    assert!(matches!(error, binrw::Error::EnumErrors { .. }));
    let result = Test::read(&mut Cursor::new(b"\x02\0\x03\0\x04")).unwrap();
    assert_eq!(result, Test::Two { a: 3, b: 4 });
}
