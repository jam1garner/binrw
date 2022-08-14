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
    }

    TestCloneArray::read(&mut binrw::io::Cursor::new(b"")).unwrap();
}

#[test]
fn phantom_data() {
    core::marker::PhantomData::<()>::read(&mut binrw::io::Cursor::new(b"")).unwrap();
}
