extern crate alloc;

#[path = "error/backtrace.rs"]
mod backtrace;
#[path = "error/backtrace_2.rs"]
mod backtrace_2;

use binrw::Error;

#[test]
fn custom_err_context() {
    use binrw::error::ContextExt;

    #[derive(Debug, Eq, PartialEq)]
    struct Oops;
    impl core::fmt::Display for Oops {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Oops")
        }
    }

    let err = Error::Custom {
        pos: 0,
        err: Box::new(Oops),
    }
    .with_message("nested oops");

    assert_eq!(err.custom_err::<Oops>(), Some(&Oops));
}

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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
fn no_seek_struct() {
    use binrw::{
        error::BacktraceFrame,
        io::{Cursor, NoSeek},
        BinRead,
    };

    #[derive(BinRead, Debug)]
    struct Test {
        #[br(assert(_a == 1))]
        _a: u32,
    }

    let mut data = NoSeek::new(Cursor::new(b"\0\0\0\0"));
    let error = Test::read_le(&mut data).expect_err("accepted bad data");
    match error {
        Error::Backtrace(bt) => {
            assert!(matches!(*bt.error, Error::Io(..)));

            match (&bt.frames[0], &bt.frames[1]) {
                (BacktraceFrame::Message(m), BacktraceFrame::Custom(e)) => {
                    assert_eq!(m, "rewinding after a failure");
                    match e.downcast_ref::<binrw::Error>() {
                        Some(binrw::Error::AssertFail { pos, .. }) => assert_eq!(*pos, 0),
                        _ => panic!("unexpected error"),
                    }
                }
                _ => panic!("unexpected error frame layout"),
            }
        }
        _ => panic!("expected backtrace"),
    }
}

#[test]
fn no_seek_data_enum() {
    use binrw::{
        error::BacktraceFrame,
        io::{Cursor, NoSeek},
        BinRead,
    };

    #[allow(dead_code)]
    #[derive(BinRead, Debug)]
    enum Test {
        #[br(magic(0u8))]
        A(#[br(assert(self_0 == 1))] u32),
        #[br(magic(1u8))]
        B(#[br(assert(self_0 == 2))] u32),
    }

    let mut data = NoSeek::new(Cursor::new(b"\0\0\0\0\0"));
    let error = Test::read_le(&mut data).expect_err("accepted bad data");

    match error {
        Error::Backtrace(bt) => {
            assert!(matches!(*bt.error, Error::Io(..)));

            match (&bt.frames[0], &bt.frames[1]) {
                (BacktraceFrame::Message(m), BacktraceFrame::Custom(e)) => {
                    assert_eq!(m, "rewinding after a failure");
                    match e.downcast_ref::<binrw::Error>() {
                        Some(binrw::Error::AssertFail { pos, .. }) => assert_eq!(*pos, 0),
                        e => panic!("unexpected error {e:?}"),
                    }
                }
                _ => panic!("unexpected error frame layout"),
            }
        }
        _ => panic!("expected backtrace"),
    }
}

#[test]
fn no_seek_unit_enum() {
    use binrw::{
        error::BacktraceFrame,
        io::{Cursor, NoSeek},
        BinRead,
    };

    #[derive(BinRead, Debug)]
    #[br(big, repr = u32)]
    enum Test {
        A = 1,
        B = 2,
        C = 3,
    }

    let mut data = NoSeek::new(Cursor::new(b"\0\0\0\0"));
    let error = Test::read_le(&mut data).expect_err("accepted bad data");

    match error {
        Error::Backtrace(bt) => {
            assert!(matches!(*bt.error, Error::Io(..)));

            match (&bt.frames[0], &bt.frames[1]) {
                (BacktraceFrame::Message(m), BacktraceFrame::Custom(e)) => {
                    assert_eq!(m, "rewinding after a failure");
                    match e.downcast_ref::<binrw::Error>() {
                        Some(binrw::Error::NoVariantMatch { pos }) => assert_eq!(*pos, 0),
                        e => panic!("unexpected error {e:?}"),
                    }
                }
                _ => panic!("unexpected error frame layout"),
            }
        }
        _ => panic!("expected backtrace"),
    }
}

#[test]
#[allow(clippy::empty_line_after_doc_comments)]
fn parse_backtrace_with_empty_comment_lines() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead)]
    struct Test {
        /// Blank next line has no whitespace…

        /// …but it is part of the same span, and needs to not crash the
        /// backtrace formatter
        _a: u32,
    }
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
    println!("{err}");
    assert_eq!(
        err,
        if cfg!(feature = "verbose-backtrace") {
            Cow::Borrowed(if cfg!(nightly) {
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
    println!("{err}");
    assert_eq!(
        err,
        if cfg!(feature = "verbose-backtrace") {
            Cow::Borrowed(if cfg!(nightly) {
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

#[test]
fn try_map_with_shadowing_box() {
    use binrw::{io::Cursor, BinRead};

    // Non-standard struct named Box intentionally shadows std::boxed::Box from the prelude
    #[allow(dead_code)]
    struct Box;

    #[allow(dead_code)]
    #[derive(BinRead, Debug)]
    struct Test {
        #[br(try_map = |_: u8| Err("Error"))]
        value: u8,
    }

    assert!(!Test::read_le(&mut Cursor::new(b"\0"))
        .expect_err("accepted bad data")
        .is_eof());
}

#[test]
fn err_context_with_shadowing_box() {
    use binrw::{io::Cursor, BinRead};

    // Non-standard struct named Box intentionally shadows std::boxed::Box from the prelude
    #[allow(dead_code)]
    struct Box;

    #[allow(dead_code)]
    #[derive(BinRead, Debug, PartialEq)]
    struct Test {
        #[br(err_context(42))]
        value: u8,
    }

    assert_eq!(
        Test::read_le(&mut Cursor::new(b"\0")).unwrap(),
        Test { value: 0 }
    );
}
