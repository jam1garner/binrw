use binrw::{BinRead, io::Cursor};

#[test]
fn boxed() {
    assert_eq!(
        Box::<u8>::read(&mut Cursor::new(b"\x03")).unwrap(),
        Box::new(3_u8)
    );
    assert!(
        Box::<(u8, u8)>::read(&mut Cursor::new(b"\x03"))
            .unwrap_err()
            .is_eof()
    );
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
    let read = binrw::helpers::count_with::<Vec<u32>, _, _, _, _>(2, binrw::helpers::read_u24);
    assert_eq!(
        read(
            &mut Cursor::new(&[0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF]),
            binrw::Endian::Little,
            (),
        )
        .expect("Vec fake-specialisation should not ignore reader function"),
        vec![0, 0xffffff]
    );
}

#[test]
fn count_huge_count_is_a_graceful_error() {
    #[derive(BinRead, Debug)]
    #[br(big)]
    struct Bytes {
        len: u64,
        #[br(count = len)]
        data: Vec<u8>,
    }

    // A count of u64::MAX used to panic in Vec::reserve_exact before any
    // byte was read; it must now be a normal parse error.
    assert!(
        Bytes::read(&mut Cursor::new([0xff; 8]))
            .expect_err("huge count should not panic")
            .is_eof()
    );

    // The integer fast path was already safe for the same count.
    #[derive(BinRead, Debug)]
    #[br(big)]
    struct Ints {
        len: u64,
        #[br(count = len)]
        data: Vec<u32>,
    }
    assert!(
        Ints::read(&mut Cursor::new([0xff; 8]))
            .expect_err("huge count should not panic")
            .is_eof()
    );

    // A count larger than the remaining input must also fail gracefully.
    assert!(
        Bytes::read(&mut Cursor::new(&[0, 0, 0, 0, 0, 0, 0, 5, 1, 2, 3][..]))
            .expect_err("short input should not panic")
            .is_eof()
    );

    // Valid and boundary reads are unchanged.
    let parsed = Bytes::read(&mut Cursor::new(&[0, 0, 0, 0, 0, 0, 0, 3, 1, 2, 3][..])).unwrap();
    assert_eq!(parsed.len, 3);
    assert_eq!(parsed.data, vec![1, 2, 3]);
    assert_eq!(
        Bytes::read(&mut Cursor::new(&[0; 8][..])).unwrap().data,
        Vec::<u8>::new()
    );
    let ints = Ints::read(&mut Cursor::new(
        &[0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2][..],
    ))
    .unwrap();
    assert_eq!(ints.len, 2);
    assert_eq!(ints.data, vec![1, 2]);

    // A legitimate large read still succeeds in one piece.
    let payload = vec![0xabu8; 1 << 20];
    let mut wire = Vec::with_capacity(8 + payload.len());
    wire.extend_from_slice(&(1_u64 << 20).to_be_bytes());
    wire.extend_from_slice(&payload);
    let parsed = Bytes::read(&mut Cursor::new(&wire)).unwrap();
    assert_eq!(parsed.len, 1 << 20);
    assert_eq!(parsed.data.len(), 1 << 20);
}
