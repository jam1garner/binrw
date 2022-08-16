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

#[rustversion::nightly]
#[cfg(all(feature = "std", not(coverage)))]
#[test]
fn show_backtrace() {
    use binrw::{io::Cursor, BinReaderExt};

    let mut x = Cursor::new(b"\0\0\0\x06");
    let err = format!(
        "{}",
        x.read_be::<backtrace::OutermostStruct>()
            .map(|_| ())
            .unwrap_err()
    );
    println!("{}", err);
    assert_eq!(err, include_str!("./error/backtrace.stderr"));
}

#[rustversion::nightly]
#[cfg(all(feature = "std", not(coverage)))]
#[test]
fn show_backtrace_2() {
    use binrw::{io::Cursor, BinReaderExt};

    let mut x = Cursor::new(b"\0\0\0\x06");
    let err = format!(
        "{}",
        x.read_be::<backtrace_2::OutermostStruct>()
            .map(|_| ())
            .unwrap_err()
    );
    println!("{}", err);
    assert_eq!(err, include_str!("./error/backtrace_2.stderr"));
}
