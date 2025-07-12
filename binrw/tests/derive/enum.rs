extern crate binrw;
use super::t;

#[test]
fn enum_assert() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(assert(b == 1))]
    enum Test {
        A {
            a: u8,
            b: u8,
        },
        #[br(assert(a == -1))]
        B {
            a: i16,
            b: u8,
        },
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\xff\xff\x01")).unwrap(),
        Test::B { a: -1, b: 1 }
    );
    <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\xff\xff\0"))
        .expect_err("accepted bad data");
    <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\0\0\x01"))
        .expect_err("accepted bad data");
}

#[test]
fn enum_assert_with_self() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(assert(self.verify()))]
    enum Test {
        A {
            a: u8,
            b: u8,
        },
        #[br(assert(self.verify_only_b()))]
        B {
            a: i16,
            b: u8,
        },
    }

    impl Test {
        fn verify(&self) -> bool {
            match self {
                Test::A { b, .. } => *b == 1,
                Test::B { a, b } => *a == -1 && *b == 1,
            }
        }

        fn verify_only_b(&self) -> bool {
            t::matches!(self, Test::B { .. })
        }
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\xff\xff\x01")).unwrap(),
        Test::B { a: -1, b: 1 }
    );
    <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\xff\xff\0"))
        .expect_err("accepted bad data");
    <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\0\0\x01"))
        .expect_err("accepted bad data");
}

#[test]
fn enum_field_args() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import(a: u8))]
    struct Foo(#[br(calc(a))] u8);

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import(a: u8))]
    enum Test {
        A {
            #[br(args(a))]
            field: Foo,
        },
    }

    let result =
        <Test as binrw::BinRead>::read_le_args(&mut binrw::io::Cursor::new(b""), (42,)).unwrap();
    t::assert_eq!(result, Test::A { field: Foo(42) });
}

#[test]
fn enum_non_copy_args() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import(a: NonCopyArg))]
    enum Test {
        A {
            #[br(calc = a.0)]
            a: u8,
        },
        B {
            #[br(calc = a.0)]
            _b: u8,
        },
    }

    #[derive(Clone)]
    struct NonCopyArg(u8);

    let result =
        <Test as binrw::BinRead>::read_le_args(&mut binrw::io::Cursor::new(b""), (NonCopyArg(1),))
            .unwrap();
    t::assert_eq!(result, Test::A { a: 1 });
}

#[test]
fn enum_calc_temp_field() {
    #[binrw::binread]
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

    let result = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\0\x04")).unwrap();
    // This also indirectly checks that `temp` is actually working since
    // compilation would fail if it weren’t due to the missing `a` property
    t::assert_eq!(result, Test::Zero { b: 4 });
}

#[test]
fn enum_endianness() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(1u16))]
        OneBig,
        #[br(little, magic(2u16))]
        TwoLittle {
            a: u16,
        },
        ThreeBig {
            a: u16,
            b: u16,
            c: u16,
        },
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x01")).unwrap(),
        Test::OneBig
    );
    let error = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x01\0"))
        .expect_err("accepted bad data");
    t::assert!(t::matches!(error, binrw::Error::EnumErrors { .. }));
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x02\0\x03\0")).unwrap(),
        Test::TwoLittle { a: 3 }
    );
    let error = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x02\x03\0"))
        .expect_err("accepted bad data");
    t::assert!(t::matches!(error, binrw::Error::EnumErrors { .. }));
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x03\x01\x00\x02\x00"))
            .unwrap(),
        Test::ThreeBig {
            a: 3,
            b: 0x100,
            c: 0x200
        }
    );
}

#[test]
fn enum_magic() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(big, magic(0x1234u16))]
    enum Test {
        #[br(magic(0u8))]
        Zero { a: u16 },
        // Fail on the second field to actually test that a rewind happens
        // to the beginning of the enum data, not to before the enum magic
        #[br(magic(1u8))]
        One { a: u16 },
    }

    let result =
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x12\x34\x01\x02\x03"))
            .unwrap();
    t::assert_eq!(result, Test::One { a: 515 });
}

#[test]
fn enum_magic_holey() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(big, magic(0x12u8))]
    enum Test {
        Wrong(#[br(magic(0x01u8))] u16),
        #[br(magic(0x12u8))]
        Right {
            a: u16,
        },
    }

    let result =
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x12\x12\x01\x02\x03"))
            .unwrap();
    t::assert_eq!(result, Test::Right { a: 0x102 });
}

#[test]
fn enum_pre_assert() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(big, import(a: bool))]
    enum Test {
        #[br(pre_assert(a))]
        A(u16),
        B(u16),
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b"\0\x01"), (true,))
            .unwrap(),
        Test::A(1)
    );
    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b"\0\x01"), (false,))
            .unwrap(),
        Test::B(1)
    );
}

#[test]
fn enum_return_all_errors() {
    #[derive(binrw::BinRead, Debug)]
    #[br(big, return_all_errors)]
    enum Test {
        #[br(magic(0u16))]
        One { _a: u16 },
        #[br(magic(1u16))]
        Two { _a: u16 },
    }

    let error = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new("\0\x01"))
        .expect_err("accepted bad data");

    match error {
        binrw::Error::EnumErrors {
            pos,
            variant_errors,
        } => {
            t::assert_eq!(pos, 0);
            t::assert_eq!(variant_errors.len(), 2);
            t::assert_eq!(variant_errors[0].0, "One");
            if let binrw::Error::BadMagic { pos, found } = &variant_errors[0].1 {
                t::assert_eq!(pos, &0);
                t::assert_eq!(&t::format!("{found:?}"), "1");
            } else {
                t::panic!("expected BadMagic; got {:?}", variant_errors[0].1);
            }
            t::assert_eq!(variant_errors[1].0, "Two");
            t::assert!(t::matches!(
                variant_errors[1].1.root_cause(),
                binrw::Error::Io(..)
            ));
        }
        _ => t::panic!("wrong error type"),
    }
}

#[test]
fn enum_rewind_on_assert() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    #[br(assert(b == 1))]
    enum Test {
        A { a: u8, b: u8 },
        B { a: u16, b: u8 },
    }

    let mut data = binrw::io::Cursor::new(b"\0\0\0\0");
    let expected = binrw::io::Seek::seek(&mut data, binrw::io::SeekFrom::Start(1)).unwrap();
    <Test as binrw::BinRead>::read_le(&mut data).expect_err("accepted bad data");
    t::assert_eq!(
        expected,
        binrw::io::Seek::stream_position(&mut data).unwrap()
    );
}

#[test]
fn enum_rewind_on_eof() {
    #[derive(binrw::BinRead, Debug)]
    enum Test {
        A {
            _a: u8,
            // Fail on the second field to actually test that a rewind happens
            // to the beginning of the enum, not just the beginning of the field
            _b: u16,
        },
    }

    let mut data = binrw::io::Cursor::new(b"\0\0\0");
    let expected = binrw::io::Seek::seek(&mut data, binrw::io::SeekFrom::Start(1)).unwrap();
    <Test as binrw::BinRead>::read_le(&mut data).expect_err("accepted bad data");
    t::assert_eq!(
        expected,
        binrw::io::Seek::stream_position(&mut data).unwrap()
    );
}

#[test]
fn enum_rewind_on_variant_assert() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    enum Test {
        #[br(assert(b == 1))]
        A { a: u8, b: u8 },
    }

    let mut data = binrw::io::Cursor::new(b"\0\0");
    let expected = binrw::io::Seek::seek(&mut data, binrw::io::SeekFrom::Start(1)).unwrap();
    <Test as binrw::BinRead>::read_le(&mut data).expect_err("accepted bad data");
    t::assert_eq!(
        expected,
        binrw::io::Seek::stream_position(&mut data).unwrap()
    );
}

#[test]
fn enum_return_unexpected_error() {
    #[derive(binrw::BinRead, Debug)]
    #[br(big, return_unexpected_error)]
    enum Test {
        #[br(magic(0u16))]
        One { _a: u16 },
        #[br(magic(1u16))]
        Two { _a: u16 },
    }

    let error = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new("\0\x01"))
        .expect_err("accepted bad data");
    t::assert!(t::matches!(error, binrw::Error::NoVariantMatch { .. }));
}

#[test]
fn mixed_enum() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(0u8))]
        Zero,
        #[br(magic(2u8))]
        Two { a: u16, b: u16 },
    }

    t::assert!(t::matches!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0")).unwrap(),
        Test::Zero
    ));
    let error = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x01"))
        .expect_err("accepted bad data");
    t::assert!(t::matches!(error, binrw::Error::EnumErrors { .. }));
    let result =
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x02\0\x03\0\x04")).unwrap();
    t::assert_eq!(result, Test::Two { a: 3, b: 4 });
}
