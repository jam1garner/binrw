use binrw::{binwrite, io::Cursor, BinWrite, Endian};
use core::convert::Infallible;

macro_rules! test_calc_args {
    ($(($calc:ident, $calc_default:ident, $directive:ident $(, $ty_wrapper:ty)?)),* $(,)?) => {
        $(#[test]
        fn $calc() {
            struct ArgsNoDefault(u8);

            #[binwrite]
            #[bw(import(args: ArgsNoDefault))]
            struct NeedsArgs(#[bw($directive = $($ty_wrapper)?(args.0))] u8);

            #[binwrite]
            struct Test {
                #[bw(args(ArgsNoDefault(1)), $directive = $($ty_wrapper)?(NeedsArgs()))]
                v: NeedsArgs,
            }

            let mut x = Cursor::new(Vec::new());

            Test {}.write_le(&mut x).unwrap();
            assert_eq!(x.into_inner(), [1]);
        }

        #[test]
        fn $calc_default() {
            struct ArgsDefault(u8);
            impl Default for ArgsDefault {
                fn default() -> Self {
                    Self(1)
                }
            }

            #[binwrite]
            #[bw(import(args: ArgsDefault))]
            struct NeedsArgs(#[bw($directive = $($ty_wrapper)?(args.0))] u8);

            #[binwrite]
            struct Test {
                #[bw($directive = $($ty_wrapper)?(NeedsArgs()))]
                v: NeedsArgs,
            }

            let mut x = Cursor::new(Vec::new());

            Test {}.write_le(&mut x).unwrap();
            assert_eq!(x.into_inner(), [1]);
        })*
    }
}

test_calc_args!(
    (calc_args, calc_args_default, calc),
    (
        try_calc_args,
        try_calc_args_default,
        try_calc,
        Ok::<_, Infallible>
    )
);

#[test]
fn calc_simple_write() {
    #[binwrite]
    struct Test {
        x: u8,

        #[bw(calc = Some(2))]
        y: Option<u16>,

        #[bw(calc = (*x as u32) + 2)]
        z: u32,
    }

    let mut x = Cursor::new(Vec::new());

    Test { x: 1 }
        .write_options(&mut x, Endian::Big, ())
        .unwrap();

    assert_eq!(x.into_inner(), [1, 0, 2, 0, 0, 0, 3]);
}

#[test]
fn calc_visibility() {
    #[binwrite]
    struct Test {
        x: u8,

        #[bw(calc = 2)]
        y: u16,

        // `y` should be visible here even though it is calculated
        #[bw(calc = y + 1)]
        z: u16,
    }

    let mut x = Cursor::new(Vec::new());

    Test { x: 1 }
        .write_options(&mut x, Endian::Big, ())
        .unwrap();

    assert_eq!(x.into_inner(), [1, 0, 2, 0, 3]);
}

#[test]
fn try_calc() {
    #[binwrite]
    #[derive(Debug, PartialEq)]
    #[bw(big, import(v: u32))]
    struct Test {
        #[bw(try_calc = <_>::try_from(v))]
        a: u16,
    }

    let mut x = Cursor::new(Vec::new());
    Test {}.write_args(&mut x, (1,)).unwrap();
    assert_eq!(x.into_inner(), b"\0\x01");
    Test {}
        .write_args(&mut Cursor::new(Vec::new()), (0x1_0000,))
        .unwrap_err();
}
