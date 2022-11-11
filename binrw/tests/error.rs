extern crate alloc;

#[path = "error/backtrace.rs"]
mod backtrace;
#[path = "error/backtrace_2.rs"]
mod backtrace_2;

use binrw::Error;

#[test]
fn custom_error_trait() {
    #[derive(Debug)]
    struct Oops;
    impl core::fmt::Display for Oops {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Oops")
        }
    }

    let err = Error::Custom {
        pos: 0,
        err: Box::new(Oops),
    };

    match err {
        Error::Custom { mut err, .. } => {
            assert!(err.is::<Oops>());
            assert!(!err.is::<i32>());
            assert!(err.downcast_ref::<Oops>().is_some());
            assert!(err.downcast_ref::<i32>().is_none());
            assert!(err.downcast_mut::<Oops>().is_some());
            assert!(err.downcast_mut::<i32>().is_none());
            match err.downcast::<i32>() {
                Ok(_) => panic!("downcast to wrong type"),
                Err(err) => assert!(err.downcast::<Oops>().is_ok()),
            }
        }
        _ => unreachable!(),
    }
}

#[test]
fn display() {
    let err = format!(
        "{}",
        Error::AssertFail {
            pos: 0x42,
            message: "Oops".into()
        }
    );
    assert!(err.contains("0x42"));
    assert!(err.contains("Oops"));

    let err = format!(
        "{}",
        Error::BadMagic {
            pos: 0x42,
            found: Box::new(57005)
        }
    );
    assert!(err.contains("0x42"));
    assert!(err.contains("57005"));

    let err = format!(
        "{}",
        Error::Io(binrw::io::Error::new(binrw::io::ErrorKind::Other, "Oops"))
    );
    assert_eq!(
        err,
        format!(
            "{}",
            binrw::io::Error::new(binrw::io::ErrorKind::Other, "Oops")
        )
    );
    #[cfg(feature = "std")]
    assert!(err.contains("Oops"));
    #[cfg(not(feature = "std"))]
    assert!(err.contains("Other"));

    let err = format!(
        "{}",
        Error::Custom {
            pos: 0x42,
            err: Box::new("Oops")
        }
    );
    assert!(err.contains("0x42"));
    assert!(err.contains("Oops"));

    let err = format!("{}", Error::NoVariantMatch { pos: 0x42 });
    assert!(err.contains("0x42"));

    let err = format!(
        "{}",
        Error::EnumErrors {
            pos: 0x42,
            variant_errors: vec![(
                "BadVariant",
                Error::AssertFail {
                    pos: 0x84,
                    message: "Oops".into()
                }
            )]
        }
    );
    assert!(err.contains("0x42"));
    assert!(err.contains("0x84"));
    assert!(err.contains("BadVariant"));
    assert!(err.contains("Oops"));
}

#[test]
fn enum_is_eol() {
    use binrw::{io::Cursor, BinRead};

    #[derive(BinRead, Debug)]
    #[br(return_all_errors)]
    enum Test {
        A(u32),
        #[br(assert(self_0 != 0))]
        B(u16),
    }

    assert!(!Test::read_le(&mut Cursor::new(b"\0\0"))
        .expect_err("accepted bad data")
        .is_eof());
    assert!(Test::read_le(&mut Cursor::new(b"\0"))
        .expect_err("accepted bad data")
        .is_eof());
}

#[test]
fn is_eof() {
    use binrw::{io::Cursor, BinRead};

    #[derive(BinRead, Debug)]
    enum A {
        A([u8; 2]),
        B([u8; 1]),
    }

    #[derive(BinRead, Debug)]
    struct Test {
        _a: A,
    }

    assert!(Test::read_le(&mut Cursor::new(b""))
        .expect_err("accepted bad data")
        .is_eof());
}

#[test]
fn not_custom_error() {
    let err = Error::AssertFail {
        pos: 0,
        message: "Oops".into(),
    };
    assert!(err.custom_err::<i32>().is_none());
}

#[test]
fn show_backtrace() {
    use alloc::borrow::Cow;
    use binrw::{io::Cursor, BinReaderExt};

    let mut x = Cursor::new(b"\x06\0\0\0");
    let err = format!(
        "{}",
        x.read_le::<backtrace::OutermostStruct>()
            .map(|_| ())
            .unwrap_err()
    );
    println!("{}", err);
    assert_eq!(
        err,
        if cfg!(feature = "verbose-backtrace") {
            Cow::Borrowed(if rustversion::cfg!(nightly) {
                include_str!("./error/backtrace_verbose_nightly.stderr")
            } else {
                include_str!("./error/backtrace_verbose.stderr")
            })
        } else {
            let bt = include_str!("./error/backtrace.stderr");
            if cfg!(feature = "std") {
                Cow::Borrowed(bt)
            } else {
                Cow::Owned(bt.replace("failed to fill whole buffer", "Simple(UnexpectedEof)"))
            }
        }
    );
}

#[test]
fn show_backtrace_2() {
    use alloc::borrow::Cow;
    use binrw::{io::Cursor, BinReaderExt};

    let mut x = Cursor::new(b"\x06\0\0\0");
    let err = format!(
        "{}",
        x.read_le::<backtrace_2::OutermostStruct>()
            .map(|_| ())
            .unwrap_err()
    );
    println!("{}", err);
    assert_eq!(
        err,
        if cfg!(feature = "verbose-backtrace") {
            Cow::Borrowed(if rustversion::cfg!(nightly) {
                include_str!("./error/backtrace_2_verbose_nightly.stderr")
            } else {
                include_str!("./error/backtrace_2_verbose.stderr")
            })
        } else {
            let bt = include_str!("./error/backtrace_2.stderr");
            if cfg!(feature = "std") {
                Cow::Borrowed(bt)
            } else {
                Cow::Owned(bt.replace("failed to fill whole buffer", "Simple(UnexpectedEof)"))
            }
        }
    );
}
