use binread::{BinRead, derive_binread};
use std::io::Cursor;

#[test]
fn enum_calc_temp_field() {
    #[derive_binread]
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
fn mixed_enum() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(0u8))] Zero,
        #[br(magic(2u8))] Two {
            a: u16,
            b: u16,
        },
    }

    assert!(matches!(Test::read(&mut Cursor::new(b"\0")).unwrap(), Test::Zero));
    let error = Test::read(&mut Cursor::new(b"\x01")).expect_err("accepted bad data");
    assert!(matches!(error, binread::Error::EnumErrors { .. }));
    let result = Test::read(&mut Cursor::new(b"\x02\0\x03\0\x04")).unwrap();
    assert_eq!(result, Test::Two { a: 3, b: 4 });
}
