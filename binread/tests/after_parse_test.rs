use binread::{BinRead, BinReaderExt, FilePtr8, io::Cursor};

#[test]
#[allow(non_snake_case)]
fn BinReaderExt_calls_after_parse() {
    let test: FilePtr8<u8> = Cursor::new([0x01, 0xFF]).read_be().unwrap();

    assert_eq!(*test, 0xFF);
}


#[derive(BinRead)]
struct Try<BR: BinRead<Args=()>>(
    #[br(try)]
    Option<BR>
);

#[test]
fn try_calls_after_parse() {
    let test: Try<FilePtr8<u8>> = Cursor::new([0x01, 0xFF]).read_be().unwrap();

    assert_eq!(*test.0.unwrap(), 0xFF)
}
