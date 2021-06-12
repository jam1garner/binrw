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
fn context() {
    use binrw::error::Context;
    use std::error::Error as StdError;

    #[derive(Debug)]
    struct Oops;
    impl core::fmt::Display for Oops {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Oops")
        }
    }
    impl StdError for Oops {}

    #[derive(Debug)]
    struct BigYikes;
    impl core::fmt::Display for BigYikes {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Big yikes")
        }
    }
    impl StdError for BigYikes {}

    let err = Error::Custom {
        pos: 0xf,
        err: Box::new(Oops),
    };

    assert!(err.source().is_none());

    let context_err = Err::<(), Error>(err)
        .context(|| "goofed".to_string())
        .unwrap_err();
    assert!(matches!(context_err, Error::Context { .. }));
    assert_eq!(
        context_err.custom_err::<String>(),
        Some(&"goofed".to_string())
    );
    assert!(matches!(context_err.custom_err::<Oops>(), Some(&Oops)));

    if let Some(Error::Custom { pos, err }) = context_err.source() {
        assert_eq!(*pos, 0xf);
        assert!(matches!(err.downcast_ref::<Oops>(), Some(&Oops)));
    } else {
        panic!("bad error returned from error source: {:?}", context_err);
    }

    #[cfg(feature = "std")]
    {
        let std_err = StdError::source(&context_err).and_then(|err| err.downcast_ref::<Error>());
        if let Some(Error::Custom { pos, err }) = std_err {
            assert_eq!(*pos, 0xf);
            assert!(matches!(err.downcast_ref::<Oops>(), Some(&Oops)));
        } else {
            panic!("bad error returned from std source: {:?}", std_err);
        }
    }

    let context_err = context_err.context(BigYikes);

    let display = format!("{}", context_err);
    assert!(display.contains("Big yikes"));
    assert!(display.contains("goofed"));
    assert!(display.contains("Oops"));
    assert!(display.contains("0xf"));
}

#[test]
fn std_context() {
    use binrw::error::Context;

    #[derive(Debug)]
    struct Oops;
    impl core::fmt::Display for Oops {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Oops")
        }
    }

    let err = Error::Custom {
        pos: 0xf,
        err: Box::new(Oops),
    };

    let context_err = Err::<(), Error>(err)
        .context(|| "lol".to_string())
        .unwrap_err();
    assert_eq!(context_err.custom_err::<String>(), Some(&"lol".to_string()));
    assert!(matches!(context_err.custom_err::<Oops>(), Some(&Oops)));
}
