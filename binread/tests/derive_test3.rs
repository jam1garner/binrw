use binread::{BinRead, BinReaderExt, io::Cursor};

#[test]
fn generic_derive() {
    #[derive(BinRead)]
    struct Test<T: BinRead<Args = ()> + Default> {
        x: [T; 3]
    }

    let mut data = Cursor::new(b"\0\x01\x02");
    let foo: Test<u8> = data.read_ne().unwrap();
    assert_eq!(foo.x, [0, 1, 2]);
}

