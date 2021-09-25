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
fn show_backtrace() {
    #![allow(dead_code)]
    use binrw::{io::Cursor, BinRead, BinReaderExt};

    #[derive(BinRead)]
    struct InnerMostStruct {
        #[br(little)]
        len: u32,

        #[br(count = len)]
        items: Vec<u32>,
    }

    #[derive(BinRead)]
    enum MiddleEnum {
        OnlyOption {
            #[br(big)]
            #[br(assert(inner.len == 3))]
            inner: InnerMostStruct,
        },
    }

    #[derive(BinRead)]
    struct MiddleStruct {
        #[br(little)]
        middle: MiddleEnum,
    }

    #[derive(BinRead)]
    struct OutermostStruct {
        #[br(little)]
        middle: MiddleStruct,
    }

    let mut x = Cursor::new(b"\0\0\0\x06");
    let err = x.read_be::<OutermostStruct>().map(|_| ()).unwrap_err();
    println!("{}", err);
}
