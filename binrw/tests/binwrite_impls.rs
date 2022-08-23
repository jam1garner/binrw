use binrw::{BinWrite, Endian};

macro_rules! compare {
    ($input:expr, $endian:expr, $output:expr) => {
        let mut output = binrw::io::Cursor::new(vec![]);
        $input
            .write_options(&mut output, &binrw::WriteOptions::new($endian), ())
            .unwrap();
        assert_eq!(output.into_inner(), $output);
    };

    ($input:expr, $output:expr) => {
        let mut output = binrw::io::Cursor::new(vec![]);
        $input.write_to(&mut output).unwrap();
        assert_eq!(output.into_inner(), $output);
    };
}

#[test]
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
    }

    compare!(
        TestCloneArray {
            _array: [ArgsNeedClone, ArgsNeedClone],
            _vec: vec![]
        },
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
fn phantom_data() {
    compare!(core::marker::PhantomData::<()>, b"");
}

#[test]
fn tuple() {
    compare!((1_u8, 2_u8), b"\x01\x02");
    compare!((1_u16, 2_u16), Endian::Big, b"\0\x01\0\x02");
    compare!((1_u16, 2_u16), Endian::Little, b"\x01\0\x02\0");
}
