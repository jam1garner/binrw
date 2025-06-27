use binrw::{io::Cursor, BinRead};

#[test]
fn derive_generic() {
    #[derive(BinRead)]
    struct Test<T: for<'a> BinRead<Args<'a> = ()> + Default> {
        a: [T; 3],
    }

    let result = Test::<u8>::read_le(&mut Cursor::new(b"\0\x01\x02")).unwrap();
    assert_eq!(result.a, [0, 1, 2]);
}

#[test]
fn derive_generic_bound() {
    #[derive(BinRead)]
    #[br(bound(T: for<'a> BinRead<Args<'a> = ()> + Default))]
    struct Test<T> {
        a: [T; 3],
    }

    let result = Test::<u8>::read_le(&mut Cursor::new(b"\0\x01\x02")).unwrap();
    assert_eq!(result.a, [0, 1, 2]);
}
