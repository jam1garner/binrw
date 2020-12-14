use binread::{BinRead, BinReaderExt, io::Cursor};

#[test]
fn generic_derive() {
    #[derive(BinRead)]
    struct Test<T: BinRead<Args = ()> + Default> {
        x: [T; 3]
    }

    let mut data = Cursor::new(b"\0\x01\x02");
    let test: Test<u8> = data.read_ne().unwrap();
    assert_eq!(test.x, [0, 1, 2]);
}

