use binread::BinRead;
use std::io::Cursor;

#[derive(BinRead, Debug)]
#[br(big, magic = 1u16)]
struct UnitStruct;

#[test]
fn unit_struct() {
    let mut test = Cursor::new(b"\x00\x01");
    UnitStruct::read(&mut test).unwrap();

    let mut test = Cursor::new(b"\x00\x00");
    UnitStruct::read(&mut test).unwrap_err();
}
