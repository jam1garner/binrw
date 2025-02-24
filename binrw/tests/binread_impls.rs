use binrw::{io::Cursor, BinRead};

#[test]
fn boxed() {
    assert_eq!(
        Box::<u8>::read(&mut Cursor::new(b"\x03")).unwrap(),
        Box::new(3_u8)
    );
    assert!(Box::<(u8, u8)>::read(&mut Cursor::new(b"\x03"))
        .unwrap_err()
        .is_eof());
}

#[test]
fn convenience_endian() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    struct Test(u16);

    #[derive(BinRead, Debug, Eq, PartialEq)]
    #[br(import(mul: u16))]
    struct TestArgs(#[br(map = |val: u16| mul * val)] u16);

    assert_eq!(Test::read_be(&mut Cursor::new(b"\0\x01")).unwrap(), Test(1));
    assert_eq!(Test::read_le(&mut Cursor::new(b"\x01\0")).unwrap(), Test(1));
    #[cfg(target_endian = "big")]
    assert_eq!(Test::read_ne(&mut Cursor::new(b"\0\x01")).unwrap(), Test(1));
    #[cfg(target_endian = "little")]
    assert_eq!(Test::read_ne(&mut Cursor::new(b"\x01\0")).unwrap(), Test(1));
    assert_eq!(
        TestArgs::read_be_args(&mut Cursor::new(b"\0\x02"), (3,)).unwrap(),
        TestArgs(6)
    );
    assert_eq!(
        TestArgs::read_le_args(&mut Cursor::new(b"\x02\0"), (3,)).unwrap(),
        TestArgs(6)
    );
    #[cfg(target_endian = "big")]
    assert_eq!(
        TestArgs::read_ne_args(&mut Cursor::new(b"\0\x02"), (3,)).unwrap(),
        TestArgs(6)
    );
    #[cfg(target_endian = "little")]
    assert_eq!(
        TestArgs::read_ne_args(&mut Cursor::new(b"\x02\0"), (3,)).unwrap(),
        TestArgs(6)
    );
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

    TestCloneArray::read_le(&mut Cursor::new(b"")).unwrap();
}

#[test]
fn non_zero() {
    assert!(matches!(
        core::num::NonZeroU8::read(&mut Cursor::new(b"\0")).expect_err("accepted bad data"),
        binrw::Error::Io(..)
    ));
    assert_eq!(
        core::num::NonZeroU8::read(&mut Cursor::new(b"\x01")).unwrap(),
        core::num::NonZeroU8::new(1).unwrap()
    );
}

#[test]
fn phantom_data() {
    core::marker::PhantomData::<()>::read(&mut Cursor::new(b"")).unwrap();
}

#[test]
fn tuple() {
    assert_eq!(
        <(u8, u8)>::read(&mut Cursor::new(b"\x01\x02")).unwrap(),
        (1, 2)
    );
}

#[test]
fn vec_u8() {
    assert!(matches!(
        Vec::<u8>::read_args(
            &mut Cursor::new(b""),
            binrw::VecArgs::builder().count(10).finalize()
        )
        .expect_err("accepted bad data"),
        binrw::Error::Io(..)
    ));
}

#[test]
fn count_with_correctness() {
    let read = binrw::helpers::count_with(2, binrw::helpers::read_u24);
    let _: Vec<u32> = read(
        &mut Cursor::new(&[0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF]),
        binrw::Endian::Little,
        (),
    )
    .expect("more than 3 bytes per u24 should not be required");
}
