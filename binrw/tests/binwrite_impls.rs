use binrw::{BinWrite, Endian};

macro_rules! compare {
    ($input:expr, $endian:expr, $output:expr) => {
        let mut output = binrw::io::Cursor::new(vec![]);
        $input.write_options(&mut output, $endian, ()).unwrap();
        assert_eq!(output.into_inner(), $output);
    };

    ($input:expr, $output:expr) => {
        let mut output = binrw::io::Cursor::new(vec![]);
        $input.write(&mut output).unwrap();
        assert_eq!(output.into_inner(), $output);
    };
}

#[test]
#[allow(unused_allocation)]
fn boxed() {
    compare!(Box::new(3_u8), b"\x03");
    compare!(Box::new(3_u16), Endian::Big, b"\0\x03");
    compare!(Box::new(3_u16), Endian::Little, b"\x03\0");
    compare!(vec![3_u8; 2].into_boxed_slice(), b"\x03\x03");
}

// This is a compile-time regression test to ensure library types allow
// cloneable arguments.
#[test]
fn clone_args() {
    #[derive(Clone)]
    struct OnlyCloneable;

    #[derive(BinWrite)]
    #[bw(import(_needs_clone: OnlyCloneable))]
    struct ArgsNeedClone;

    #[derive(BinWrite)]
    struct TestCloneArray {
        // Test for `[T; N]::Args`
        #[bw(args(OnlyCloneable))]
        _array: [ArgsNeedClone; 2],

        // Test for `Vec<T>::Args`
        #[bw(args(OnlyCloneable))]
        _vec: Vec<ArgsNeedClone>,

        // Test for `(T, T)::Args`
        #[bw(args(OnlyCloneable))]
        _tuple: (ArgsNeedClone, ArgsNeedClone),
    }

    compare!(
        TestCloneArray {
            _array: [ArgsNeedClone, ArgsNeedClone],
            _vec: vec![],
            _tuple: (ArgsNeedClone, ArgsNeedClone),
        },
        Endian::Big,
        b""
    );
}

#[test]
fn non_zero() {
    compare!(core::num::NonZeroU8::new(1).unwrap(), b"\x01");
    compare!(
        core::num::NonZeroU16::new(1).unwrap(),
        Endian::Big,
        b"\0\x01"
    );
    compare!(
        core::num::NonZeroU16::new(1).unwrap(),
        Endian::Little,
        b"\x01\0"
    );
}

#[test]
fn native_endian() {
    #[derive(BinWrite)]
    struct Test(u16);

    #[derive(BinWrite)]
    #[bw(import(mul: u16))]
    struct TestArgs(#[bw(map = |val| mul * *val)] u16);

    let mut output = binrw::io::Cursor::new(vec![]);
    Test(1).write_ne(&mut output).unwrap();
    #[cfg(target_endian = "big")]
    assert_eq!(output.into_inner(), b"\0\x01");
    #[cfg(target_endian = "little")]
    assert_eq!(output.into_inner(), b"\x01\0");

    let mut output = binrw::io::Cursor::new(vec![]);
    TestArgs(2).write_ne_args(&mut output, (2,)).unwrap();
    #[cfg(target_endian = "big")]
    assert_eq!(output.into_inner(), b"\0\x04");
    #[cfg(target_endian = "little")]
    assert_eq!(output.into_inner(), b"\x04\0");
}

#[test]
fn option() {
    compare!(Some(1_i32), Endian::Big, b"\0\0\0\x01");
    compare!(None::<i32>, Endian::Big, b"");
}

#[test]
fn phantom_data() {
    compare!(core::marker::PhantomData::<()>, b"");
}

#[test]
fn tuple() {
    compare!((1_u8, 2_u8), b"\x01\x02");
    compare!((1_u16, 2_u16), Endian::Big, b"\0\x01\0\x02");
    compare!((1_u16, 2_u16), Endian::Little, b"\x01\0\x02\0");
}

#[test]
fn vec_i8() {
    let mut output = binrw::io::Cursor::new(vec![]);
    vec![-1_i8; 4].write(&mut output).unwrap();
    assert_eq!(output.into_inner(), b"\xff\xff\xff\xff");
}
