extern crate binrw;
use super::t;

#[test]
fn correct_args_type_set() {
    #[derive(binrw::BinWrite)]
    #[bw(import { _x: u32, _y: u8 })]
    struct Test {}

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_le_args(&Test {}, &mut x, binrw::args! { _x: 3, _y: 2 }).unwrap();
}

#[test]
fn gat_list() {
    #[derive(binrw::BinWrite)]
    #[bw(little, import(borrowed: &u8))]
    struct Test {
        #[bw(map = |a| *a + *borrowed)]
        a: u8,
    }

    let mut out = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write_args(&Test { a: 0 }, &mut out, (&1_u8,)).unwrap();

    t::assert_eq!(out.into_inner(), b"\x01");
}

#[test]
fn gat_named() {
    #[derive(binrw::BinWrite)]
    #[bw(little, import { borrowed: &u8 })]
    struct Test {
        #[bw(map = |a| *a + *borrowed)]
        a: u8,
    }

    let mut out = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write_args(&Test { a: 0 }, &mut out, binrw::args! { borrowed: &1_u8, })
        .unwrap();

    t::assert_eq!(out.into_inner(), b"\x01");
}

#[test]
fn gat_raw() {
    #[derive(binrw::BinWrite)]
    #[bw(little, import_raw(borrowed: &u8))]
    struct Test {
        #[bw(map = |a| *a + *borrowed)]
        a: u8,
    }

    let mut out = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write_args(&Test { a: 0 }, &mut out, &1_u8).unwrap();

    t::assert_eq!(out.into_inner(), b"\x01");
}

#[test]
fn shadowed_imports() {
    #[derive(binrw::BinWrite)]
    #[bw(import { x: u32 })]
    struct Test {
        x: u8,
    }

    let mut out = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write_le_args(&Test { x: 1 }, &mut out, binrw::args! { x: 2 }).unwrap();
    t::assert_eq!(out.into_inner(), b"\x01");
}

#[test]
fn usable_args() {
    #[binrw::binrw]
    #[bw(import { x: u32, _y: u8 })]
    struct Test {
        #[br(temp, ignore)]
        #[bw(calc = x)]
        x_copy: u32,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_le_args(&Test {}, &mut x, binrw::args! { x: 3, _y: 2 }).unwrap();
}
