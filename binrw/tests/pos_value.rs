extern crate alloc;

use alloc::format;
use binrw::{io::Cursor, BinRead, BinReaderExt, BinWrite, PosValue};

#[test]
fn pos_value() {
    #[derive(BinRead, BinWrite, Default)]
    struct MyType {
        a: u16,
        b: PosValue<u8>,
    }

    let mut val: MyType = Cursor::new(b"\xFF\xFE\xFD").read_be::<MyType>().unwrap();
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

    let mut output = Vec::new();
    val.write_be(&mut Cursor::new(&mut output)).unwrap();

    assert_eq!(output, b"\xFF\xFE\x01");
    let default_val = MyType::default();
    assert_eq!(default_val.a, u16::default());
    assert_eq!(*default_val.b, u8::default());
    assert_eq!(default_val.b.pos, u64::default());

    let from = MyType {
        a: val.a,
        b: (*val.b).into(),
    };
    assert_eq!(from.a, val.a);
    assert_eq!(from.b, *val.b);
}
