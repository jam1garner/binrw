extern crate binrw;
use super::t;

macro_rules! test_calc_args {
    ($(($calc:ident, $calc_default:ident, $directive:ident $(, $ty_wrapper:ty)?)),* $(,)?) => {
        $(#[test]
        fn $calc() {
            struct ArgsNoDefault(u8);

            #[binrw::binwrite]
            #[bw(import(args: ArgsNoDefault))]
            struct NeedsArgs(#[bw($directive = $($ty_wrapper)?(args.0))] u8);

            #[binrw::binwrite]
            struct Test {
                #[bw(args(ArgsNoDefault(1)), $directive = $($ty_wrapper)?(NeedsArgs()))]
                v: NeedsArgs,
            }

            let mut x = binrw::io::Cursor::new(t::Vec::new());

            binrw::BinWrite::write_le(&Test {}, &mut x).unwrap();
            t::assert_eq!(x.into_inner(), [1]);
        }

        #[test]
        fn $calc_default() {
            struct ArgsDefault(u8);
            impl t::Default for ArgsDefault {
                fn default() -> Self {
                    Self(1)
                }
            }

            #[binrw::binwrite]
            #[bw(import(args: ArgsDefault))]
            struct NeedsArgs(#[bw($directive = $($ty_wrapper)?(args.0))] u8);

            #[binrw::binwrite]
            struct Test {
                #[bw($directive = $($ty_wrapper)?(NeedsArgs()))]
                v: NeedsArgs,
            }

            let mut x = binrw::io::Cursor::new(t::Vec::new());

            binrw::BinWrite::write_le(&mut Test {}, &mut x).unwrap();
            t::assert_eq!(x.into_inner(), [1]);
        })*
    }
}

test_calc_args!(
    (calc_args, calc_args_default, calc),
    (
        try_calc_args,
        try_calc_args_default,
        try_calc,
        t::Ok::<_, ::core::convert::Infallible>
    )
);

#[test]
fn calc_simple_write() {
    #[binrw::binwrite]
    struct Test {
        x: u8,

        #[bw(calc = t::Some(2))]
        y: t::Option<u16>,

        #[bw(calc = (*x as u32) + 2)]
        z: u32,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(&Test { x: 1 }, &mut x, binrw::Endian::Big, ()).unwrap();

    t::assert_eq!(x.into_inner(), [1, 0, 2, 0, 0, 0, 3]);
}

#[test]
fn calc_visibility() {
    #[binrw::binwrite]
    struct Test {
        x: u8,

        #[bw(calc = 2)]
        y: u16,

        // `y` should be visible here even though it is calculated
        #[bw(calc = y + 1)]
        z: u16,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(&Test { x: 1 }, &mut x, binrw::Endian::Big, ()).unwrap();

    t::assert_eq!(x.into_inner(), [1, 0, 2, 0, 3]);
}

#[test]
fn try_calc() {
    #[binrw::binwrite]
    #[derive(Debug, PartialEq)]
    #[bw(big, import(v: u32))]
    struct Test {
        #[bw(try_calc = ::core::convert::TryFrom::try_from(v))]
        a: u16,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write_args(&Test {}, &mut x, (1,)).unwrap();
    t::assert_eq!(x.into_inner(), b"\0\x01");
    binrw::BinWrite::write_args(
        &Test {},
        &mut binrw::io::Cursor::new(t::Vec::new()),
        (0x1_0000,),
    )
    .unwrap_err();
}
