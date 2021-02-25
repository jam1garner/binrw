use binread::{BinRead, io::Cursor};
use core::convert::TryInto;

#[test]
fn map_closure() {
    #[derive(BinRead, Debug)]
    #[br(big, import(extra: u8))]
    struct Test {
        // Capturing a variable in the closure ensures that a closure type is
        // actually tested, since a closure that does not capture is a plain
        // function, not a functor
        #[br(map = |value: u8| (value + extra).into())]
        a: i16,
    }

    let result = Test::read_args(&mut Cursor::new("\x01"), (5, )).unwrap();
    assert_eq!(result.a, 6);
}

#[test]
fn map_expr() {
    #[derive(BinRead, Debug)]
    #[br(big)]
    struct Test {
        #[br(map = make_map(1))]
        a: i16,
    }

    fn make_map(extra: u8) -> impl Fn(u8) -> i16 {
        move |value: u8| { (value + extra).into() }
    }

    let result = Test::read(&mut Cursor::new("\x01")).unwrap();
    assert_eq!(result.a, 2);
}

#[test]
fn try_map() {
    #[derive(BinRead, Debug)]
    #[br(big)]
    struct Test {
        #[br(try_map = |x: i32| { x.try_into() })]
        a: i16,
    }

    let result = Test::read(&mut Cursor::new(b"\xff\xff\xff\xff")).unwrap();
    assert_eq!(result.a, -1);
    let error = Test::read(&mut Cursor::new(b"\x7f\0\0\0")).expect_err("accepted bad data");
    error.custom_err::<<i32 as ::core::convert::TryInto<i16>>::Error>().expect("wrong error type");
}
