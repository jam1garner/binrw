use binrw::BinRead;

#[test]
fn boxed() {
    assert_eq!(
        Box::<u8>::read(&mut binrw::io::Cursor::new(b"\x03")).unwrap(),
        Box::new(3_u8)
    );
    assert!(Box::<u16>::read(&mut binrw::io::Cursor::new(b"\x03"))
        .unwrap_err()
        .is_eof());
}

// This is a compile-time regression test to ensure library types allow
// cloneable arguments.
#[test]
fn clone_args() {
    #[derive(Clone)]
    struct OnlyCloneable;

    #[derive(BinRead)]
    #[br(import(_needs_clone: OnlyCloneable))]
    struct ArgsNeedClone;

    #[derive(BinRead)]
    struct TestCloneArray {
        // Test for `[T; N]::Args`
        #[br(args(OnlyCloneable))]
        _array: [ArgsNeedClone; 35],

        // Test for `Vec<T>::Args`
        #[br(count = 4, args { inner: (OnlyCloneable,) })]
        _vec: Vec<ArgsNeedClone>,

        // Test for `(T, T)::Args`
        #[br(args(OnlyCloneable))]
        _tuple: (ArgsNeedClone, ArgsNeedClone),
    }

    TestCloneArray::read(&mut binrw::io::Cursor::new(b"")).unwrap();
}

#[test]
fn non_zero() {
    assert!(matches!(
        core::num::NonZeroU8::read(&mut binrw::io::Cursor::new(b"\0"))
            .expect_err("accepted bad data"),
        binrw::Error::Io(..)
    ));
    assert_eq!(
        core::num::NonZeroU8::read(&mut binrw::io::Cursor::new(b"\x01")).unwrap(),
        core::num::NonZeroU8::new(1).unwrap()
    );
}

#[test]
fn phantom_data() {
    core::marker::PhantomData::<()>::read(&mut binrw::io::Cursor::new(b"")).unwrap();
}

#[test]
fn tuple() {
    assert_eq!(
        <(u8, u8)>::read(&mut binrw::io::Cursor::new(b"\x01\x02")).unwrap(),
        (1, 2)
    );
}

#[test]
fn vec_u8() {
    assert!(matches!(
        Vec::<u8>::read_args(
            &mut binrw::io::Cursor::new(b""),
            binrw::VecArgs::builder().count(10).finalize()
        )
        .expect_err("accepted bad data"),
        binrw::Error::Io(..)
    ));
}
