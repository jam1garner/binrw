use binrw::{io::Cursor, BinRead, BinReaderExt, FilePtr8};

#[test]
#[allow(non_snake_case)]
fn BinReaderExt_calls_after_parse() {
    let test: FilePtr8<u8> = Cursor::new([0x01, 0xFF]).read_be().unwrap();

    assert_eq!(*test, 0xFF);
}

#[test]
fn try_calls_after_parse() {
    #[derive(BinRead)]
    struct Try<BR>(#[br(try)] Option<BR>)
    where
        BR: BinRead,
        for<'a> BR::Args<'a>: Default + 'static;

    let test: Try<FilePtr8<u8>> = Cursor::new([0x01, 0xFF]).read_be().unwrap();

    assert_eq!(*test.0.unwrap(), 0xFF)
}

#[test]
fn tuple_calls_after_parse() {
    let test: (FilePtr8<u8>, FilePtr8<u8>) = Cursor::new([2, 3, 0xFF, 0xEE]).read_be().unwrap();
    assert_eq!(*test.0, 0xFF);
    assert_eq!(*test.1, 0xEE);
}
