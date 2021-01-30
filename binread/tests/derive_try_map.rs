use binread::{BinRead, io::Cursor};
use std::convert::TryInto;

#[derive(BinRead, Debug)]
#[br(big)]
struct Test {
    #[br(try_map = |x: i32| { x.try_into() })]
    x: i16,
}

#[test]
fn derive_try_map_success() {
    let mut data = Cursor::new(b"\xff\xff\xff\xff");
    let test = Test::read(&mut data).expect("Map should have succeeded");
    assert_eq!(test.x, -1);
}

#[test]
fn derive_try_map_fail() {
    let mut data = Cursor::new(b"\x7f\0\0\0");
    let err = Test::read(&mut data).expect_err("Map should have failed");
    err.custom_err::<<i32 as ::core::convert::TryInto<i16>>::Error>().expect("Map error should come from the closure");
}

#[derive(BinRead, Debug)]
#[br(big, import(extra: u8))]
struct TestCapture {
    #[br(map = |value: u8| value + extra)]
    x: u8
}

#[test]
fn derive_map() {
    let mut data = Cursor::new("\x01");
    let test = TestCapture::read_args(&mut data, (5, )).expect("Map should have succeeded");
    assert_eq!(test.x, 6);
}
