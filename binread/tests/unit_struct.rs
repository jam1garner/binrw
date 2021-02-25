use binread::BinRead;
use std::io::Cursor;

#[test]
fn unit_struct_magic() {
    #[derive(BinRead, Debug)]
    #[br(big, magic = 1u16)]
    struct Test;

    Test::read(&mut Cursor::new(b"\x00\x01")).unwrap();
    let error = Test::read(&mut Cursor::new(b"\x00\x00")).expect_err("accepted bad data");
    assert!(matches!(error, binread::Error::BadMagic { .. }));
}
