extern crate alloc;

use alloc::format;
use binrw::{io::Cursor, BinRead, BinReaderExt, PosValue};

#[test]
fn pos_value() {
    #[derive(BinRead)]
    struct MyType {
        a: u16,
        b: PosValue<u8>,
    }

    let mut val = Cursor::new(b"\xFF\xFE\xFD").read_be::<MyType>().unwrap();
    assert_eq!(val.a, 0xFFFE);
    assert_eq!(val.b.pos, 2);
    assert_eq!(*val.b, 0xFD);
    assert_eq!(val.b, 0xFDu8);

    *val.b = 1u8;
    assert_eq!(*val.b, 1);
    assert_eq!(format!("{:?}", val.b), "1");
    let clone = val.b.clone();
    assert_eq!(*clone, *val.b);
    assert_eq!(clone.pos, val.b.pos);
}
