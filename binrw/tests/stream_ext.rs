use binrw::{binwrite, BinReaderExt, BinWriterExt, VecArgs};

#[test]
#[allow(non_snake_case)]
fn BinReaderExt() {
    let mut data = binrw::io::Cursor::new(
        b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12",
    );
    assert_eq!(data.read_be::<u16>().unwrap(), 0x102);
    assert_eq!(data.read_le::<u16>().unwrap(), 0x403);
    #[cfg(target_endian = "little")]
    assert_eq!(data.read_ne::<u16>().unwrap(), 0x605);
    #[cfg(target_endian = "big")]
    assert_eq!(data.read_ne::<u16>().unwrap(), 0x506);
    assert_eq!(
        data.read_be_args::<Vec<u16>>(VecArgs::builder().count(2).finalize())
            .unwrap(),
        vec![0x708, 0x90a]
    );
    assert_eq!(
        data.read_le_args::<Vec<u16>>(VecArgs::builder().count(2).finalize())
            .unwrap(),
        vec![0xc0b, 0xe0d]
    );
    #[cfg(target_endian = "little")]
    assert_eq!(
        data.read_ne_args::<Vec<u16>>(VecArgs::builder().count(2).finalize())
            .unwrap(),
        vec![0x100f, 0x1211]
    );
    #[cfg(target_endian = "big")]
    assert_eq!(
        data.read_ne_args::<Vec<u16>>(VecArgs::builder().count(2).finalize())
            .unwrap(),
        vec![0xf10, 0x1112]
    );
}

#[test]
#[allow(non_snake_case)]
fn BinWriterExt() {
    #[binwrite]
    #[bw(import(a: u16))]
    struct Argsy(u16, #[bw(calc = a)] u16);

    let mut data = binrw::io::Cursor::new(Vec::new());
    data.write_be::<u16>(&0x102).unwrap();
    data.write_le::<u16>(&0x403).unwrap();
    #[cfg(target_endian = "little")]
    data.write_ne::<u16>(&0x605).unwrap();
    #[cfg(target_endian = "big")]
    data.write_ne::<u16>(&0x506).unwrap();
    data.write_be_args(&Argsy(0x708), (0x90a,)).unwrap();
    data.write_le_args(&Argsy(0xc0b), (0xe0d,)).unwrap();
    #[cfg(target_endian = "little")]
    data.write_ne_args(&Argsy(0x100f), (0x1211,)).unwrap();
    #[cfg(target_endian = "big")]
    data.write_ne_args(&Argsy(0xf10), (0x1112,)).unwrap();
    assert_eq!(
        data.into_inner(),
        b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12"
    );
}
