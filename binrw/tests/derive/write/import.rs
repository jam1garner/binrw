use binrw::{binrw, io::Cursor, BinWrite};

#[test]
fn correct_args_type_set() {
    #[derive(BinWrite)]
    #[bw(import { _x: u32, _y: u8 })]
    struct Test {}

    let mut x = Cursor::new(Vec::new());

    Test {}
        .write_le_args(&mut x, binrw::args! { _x: 3, _y: 2 })
        .unwrap();
}

#[test]
fn gat_list() {
    #[derive(BinWrite)]
    #[bw(little, import(borrowed: &u8))]
    struct Test {
        #[bw(map = |a| *a + *borrowed)]
        a: u8,
    }

    let mut out = Cursor::new(Vec::new());
    Test { a: 0 }.write_args(&mut out, (&1_u8,)).unwrap();

    assert_eq!(out.into_inner(), b"\x01");
}

#[test]
fn gat_named() {
    #[derive(BinWrite)]
    #[bw(little, import { borrowed: &u8 })]
    struct Test {
        #[bw(map = |a| *a + *borrowed)]
        a: u8,
    }

    let mut out = Cursor::new(Vec::new());
    Test { a: 0 }
        .write_args(&mut out, binrw::args! { borrowed: &1_u8, })
        .unwrap();

    assert_eq!(out.into_inner(), b"\x01");
}

#[test]
fn gat_raw() {
    #[derive(BinWrite)]
    #[bw(little, import_raw(borrowed: &u8))]
    struct Test {
        #[bw(map = |a| *a + *borrowed)]
        a: u8,
    }

    let mut out = Cursor::new(Vec::new());
    Test { a: 0 }.write_args(&mut out, &1_u8).unwrap();

    assert_eq!(out.into_inner(), b"\x01");
}

#[test]
fn shadowed_imports() {
    #[derive(BinWrite)]
    #[bw(import { x: u32 })]
    struct Test {
        x: u8,
    }

    let mut out = Cursor::new(Vec::new());
    Test { x: 1 }
        .write_le_args(&mut out, binrw::args! { x: 2 })
        .unwrap();
    assert_eq!(out.into_inner(), b"\x01");
}

#[test]
fn usable_args() {
    #[binrw]
    #[bw(import { x: u32, _y: u8 })]
    struct Test {
        #[br(temp, ignore)]
        #[bw(calc = x)]
        x_copy: u32,
    }

    let mut x = Cursor::new(Vec::new());

    Test {}
        .write_le_args(&mut x, binrw::args! { x: 3, _y: 2 })
        .unwrap();
}
