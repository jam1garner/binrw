use binread::{BinRead, io::Cursor};

#[test]
fn derive_generic() {
    #[derive(BinRead)]
    struct Test<T: BinRead<Args = ()> + Default> {
        a: [T; 3],
    }

    let result = Test::<u8>::read(&mut Cursor::new(b"\0\x01\x02")).unwrap();
    assert_eq!(result.a, [ 0, 1, 2 ]);
}
