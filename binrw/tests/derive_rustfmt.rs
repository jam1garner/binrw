// This should actually be part of the derive test suite, but it is not possibl
// to combine `#[rustfmt]` with `#![no_implicit_prelude]` due to
// <https://github.com/rust-lang/rust/issues/106518>
#[test]
fn named_args_trailing_commas() {
    use binrw::{io::Cursor, BinRead};

    #[rustfmt::skip]
    #[derive(BinRead, Debug, PartialEq)]
    struct Test {
        count: u8,
        #[br(args {
            count: count.into(),
            inner: ItemBinReadArgs { count: 2, },
        })]
        items: Vec<Item>,
    }

    #[rustfmt::skip]
    #[derive(BinRead, Debug, PartialEq)]
    #[br(import { count: usize, })]
    struct Item(#[br(args { count, })] Vec<u8>);

    assert_eq!(
        Test::read_le(&mut Cursor::new(b"\x03\x04\0\x05\0\x06\0")).unwrap(),
        Test {
            count: 3,
            items: vec![Item(vec![4, 0]), Item(vec![5, 0]), Item(vec![6, 0])]
        }
    );
}
