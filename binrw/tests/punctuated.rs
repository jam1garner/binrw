#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::format;
use binrw::{io::Cursor, punctuated::Punctuated, BinRead, BinReaderExt};

#[derive(BinRead, Clone, Copy, Debug)]
#[br(magic = 1u8)]
struct One;

#[derive(BinRead, Clone, Copy, Debug)]
#[br(magic = 2u8)]
struct Two;

#[derive(BinRead)]
struct PunctuatedTest {
    count: u8,

    #[br(count = count)]
    #[br(parse_with = Punctuated::<One, Two>::separated)]
    list: Punctuated<One, Two>,
}

#[derive(BinRead)]
struct PunctuatedTestTrailing {
    count: u8,

    #[br(count = count)]
    #[br(parse_with = Punctuated::<One, Two>::separated_trailing)]
    list: Punctuated<One, Two>,
}

// TODO: move to UI tests?
// #[derive(BinRead)]
// struct MissingCount {
//     #[br(parse_with = Punctuated::separated)]
//     _list: Punctuated<One, Two>,
// }
//
// #[derive(BinRead)]
// struct MissingCountTrailing {
//     #[br(parse_with = Punctuated::separated_trailing)]
//     _list: Punctuated<One, Two>,
// }

const TEST_DATA: &[u8] = b"\x03\x01\x02\x01\x02\x01";
const TEST_DATA_TRAILING: &[u8] = b"\x03\x01\x02\x01\x02\x01\x02";

#[test]
fn punctuated() {
    let mut x = Cursor::new(TEST_DATA);

    let y: PunctuatedTest = x.read_be().unwrap();

    assert_eq!(y.count, 3);
    assert_eq!(y.list.len(), 3);

    // This behavior may be reworked later
    assert_eq!(format!("{:?}", y.list), "[One, One, One]");
    assert_eq!(format!("{:?}", y.list.into_values()), "[One, One, One]");
}

#[test]
fn punctuated_trailing() {
    let mut x = Cursor::new(TEST_DATA_TRAILING);

    let mut y: PunctuatedTestTrailing = x.read_be().unwrap();

    assert_eq!(y.count, 3);
    assert_eq!(y.list.len(), 3);

    let y = &mut *y.list;
    y[0] = y[1];
}

// TODO: move to UI tests?
// #[test]
// #[should_panic]
// fn missing_count() {
//     let mut x = Cursor::new(TEST_DATA);
//
//     let _: MissingCount = x.read_be().unwrap();
// }
//
// #[test]
// #[should_panic]
// fn missing_count_trailing() {
//     let mut x = Cursor::new(TEST_DATA);
//
//     let _: MissingCountTrailing = x.read_be().unwrap();
// }
