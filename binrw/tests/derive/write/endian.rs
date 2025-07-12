extern crate binrw;
use super::t;

#[derive(binrw::BinWrite)]
struct TestEndian {
    x: u16,

    #[bw(little)]
    y: u16,

    #[bw(is_big = true)]
    z: u32,

    #[bw(is_big = false)]
    not_z: u32,
}

#[test]
fn write_endian() {
    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_be(
        &TestEndian {
            x: 1,
            y: 2,
            z: 3,
            not_z: 3,
        },
        &mut x,
    )
    .unwrap();

    t::assert_eq!(x.into_inner(), [0, 1, 2, 0, 0, 0, 0, 3, 3, 0, 0, 0]);
}

#[test]
fn top_level_endian() {
    #[derive(binrw::BinWrite)]
    #[bw(is_big = true)]
    struct Test {
        #[bw(big)] // <-- will be ignored
        little: TestLittle,

        big: TestInheritBig,
    }

    #[derive(binrw::BinWrite)]
    #[bw(little)]
    struct TestLittle {
        x: u16,
        y: u32,
    }

    #[derive(binrw::BinWrite)]
    struct TestInheritBig {
        x: u16,
        y: u32,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write(
        &Test {
            little: TestLittle { x: 1, y: 2 },
            big: TestInheritBig { x: 3, y: 4 },
        },
        &mut x,
    )
    .unwrap();

    t::assert_eq!(x.into_inner(), [1, 0, 2, 0, 0, 0, 0, 3, 0, 0, 0, 4]);
}
