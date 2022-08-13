use binrw::{io::Cursor, BinRead};
use core::convert::TryInto;

#[test]
fn map_closure() {
    #[derive(BinRead, Debug)]
    #[br(big, import(extra: u8))]
    struct Test {
        // Capturing a variable in the closure ensures that a closure type is
        // actually tested, since a closure that does not capture is a plain
        // function, not a functor
        #[br(map = |value: u8| (value + extra).into())]
        a: i16,
    }

    let result = Test::read_args(&mut Cursor::new("\x01"), (5,)).unwrap();
    assert_eq!(result.a, 6);
}

#[test]
fn map_expr() {
    #[derive(BinRead, Debug)]
    #[br(big)]
    struct Test {
        #[br(map = make_map(1))]
        a: i16,
    }

    fn make_map(extra: u8) -> impl Fn(u8) -> i16 {
        move |value: u8| (value + extra).into()
    }

    let result = Test::read(&mut Cursor::new("\x01")).unwrap();
    assert_eq!(result.a, 2);
}

#[test]
fn map_repr_enum() {
    #[derive(BinRead, Debug, PartialEq)]
    #[br(repr = u8)]
    enum Test {
        SubTest(u8),
    }

    impl From<u8> for Test {
        fn from(u: u8) -> Self {
            Self::SubTest(u)
        }
    }

    let result = Test::read(&mut Cursor::new("\x01")).unwrap();
    assert_eq!(result, Test::SubTest(1));
}

#[test]
fn map_repr_enum_variant() {
    #[derive(BinRead, Debug, PartialEq)]
    enum Test {
        SubTest(#[br(repr = u8)] SubTest),
    }

    #[derive(Debug, PartialEq)]
    struct SubTest(u8);

    impl From<u8> for SubTest {
        fn from(u: u8) -> Self {
            Self(u)
        }
    }

    let result = Test::read(&mut Cursor::new("\x01")).unwrap();
    assert_eq!(result, Test::SubTest(SubTest(1)));
}

#[test]
fn map_repr_struct() {
    #[derive(BinRead, Debug)]
    #[br(repr = u8)]
    struct Test {
        a: u8,
    }

    impl From<u8> for Test {
        fn from(a: u8) -> Self {
            Self { a }
        }
    }

    let result = Test::read(&mut Cursor::new("\x01")).unwrap();
    assert_eq!(result.a, 1);
}

#[test]
fn map_repr_struct_field() {
    #[derive(BinRead, Debug)]
    #[br(big)]
    struct Test {
        #[br(repr = u8)]
        a: SubTest,
    }

    #[derive(Debug)]
    struct SubTest {
        a: u8,
    }

    impl From<u8> for SubTest {
        fn from(a: u8) -> Self {
            Self { a }
        }
    }

    let result = Test::read(&mut Cursor::new("\x01")).unwrap();
    assert_eq!(result.a.a, 1);
}

#[test]
fn map_struct() {
    #[derive(BinRead, Debug)]
    #[br(map = Self::from_bytes)]
    struct Test {
        a: i16,
    }

    impl Test {
        fn from_bytes(bytes: [u8; 2]) -> Self {
            Self {
                a: i16::from(bytes[0]) | (i16::from(bytes[1]) << 8),
            }
        }
    }

    let result = Test::read(&mut Cursor::new(b"\0\x01")).unwrap();
    assert_eq!(result.a, 256);
}

#[test]
fn try_map_field() {
    #[derive(BinRead, Debug)]
    #[br(big)]
    struct Test {
        #[br(try_map = |x: i32| { x.try_into() })]
        a: i16,
    }

    let result = Test::read(&mut Cursor::new(b"\xff\xff\xff\xff")).unwrap();
    assert_eq!(result.a, -1);
    let error = Test::read(&mut Cursor::new(b"\x7f\0\0\0")).expect_err("accepted bad data");
    assert!(matches!(error, binrw::Error::Custom { pos: 0, .. }));
    error
        .custom_err::<<i32 as ::core::convert::TryInto<i16>>::Error>()
        .expect("wrong error type");
}

#[test]
fn try_map_struct() {
    #[derive(BinRead, Debug)]
    #[br(try_map = Self::from_bytes)]
    struct Test {
        a: i16,
    }

    #[derive(Debug)]
    struct Oops;
    impl core::fmt::Display for Oops {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Debug::fmt(self, f)
        }
    }

    impl Test {
        fn from_bytes(bytes: [u8; 2]) -> Result<Self, Oops> {
            if bytes[0] == 0 {
                Ok(Self {
                    a: i16::from(bytes[0]) | (i16::from(bytes[1]) << 8),
                })
            } else {
                Err(Oops)
            }
        }
    }

    let result = Test::read(&mut Cursor::new(b"\0\x01")).unwrap();
    assert_eq!(result.a, 256);
    let error = Test::read(&mut Cursor::new(b"\x01\0")).expect_err("accepted bad data");
    error.custom_err::<Oops>().unwrap();
}

#[test]
fn try_map_struct_closure() {
    #[derive(BinRead, Debug)]
    #[br(try_map = |a| { Self::from_bytes(a) })]
    struct Test {
        a: i16,
    }

    #[derive(Debug)]
    struct Oops;
    impl core::fmt::Display for Oops {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Debug::fmt(self, f)
        }
    }

    impl Test {
        fn from_bytes(bytes: [u8; 2]) -> Result<Self, Oops> {
            if bytes[0] == 0 {
                Ok(Self {
                    a: i16::from(bytes[0]) | (i16::from(bytes[1]) << 8),
                })
            } else {
                Err(Oops)
            }
        }
    }

    let result = Test::read(&mut Cursor::new(b"\0\x01")).unwrap();
    assert_eq!(result.a, 256);
    let error = Test::read(&mut Cursor::new(b"\x01\0")).expect_err("accepted bad data");
    error.custom_err::<Oops>().unwrap();
}
