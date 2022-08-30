#[test]
fn read_endian() {
    use binrw::{
        meta::{EndianKind, ReadEndian},
        BinRead, Endian,
    };

    #[derive(BinRead)]
    #[br(big)]
    struct Big(u16);

    #[derive(BinRead)]
    #[br(little)]
    struct Little(u16);

    #[derive(BinRead)]
    #[br(is_little = true)]
    struct Runtime(u8);

    assert_eq!(Big::ENDIAN.endian(), Some(Endian::Big));
    assert_eq!(Little::ENDIAN.endian(), Some(Endian::Little));
    assert_eq!(u8::ENDIAN, EndianKind::None);
    assert_eq!(u8::ENDIAN.endian(), None);
    assert_eq!(Runtime::ENDIAN, EndianKind::Runtime);
    assert_eq!(Runtime::ENDIAN.endian(), None);
    assert_eq!(<(u8, Big)>::ENDIAN, EndianKind::Mixed);
    assert_eq!(<(u8, Big)>::ENDIAN.endian(), None);
}

#[test]
fn write_endian() {
    use binrw::{
        meta::{EndianKind, WriteEndian},
        BinWrite, Endian,
    };

    #[derive(BinWrite)]
    #[bw(big)]
    struct Big(u16);

    #[derive(BinWrite)]
    #[bw(little)]
    struct Little(u16);

    #[derive(BinWrite)]
    #[bw(is_big = false)]
    struct Runtime(u8);

    assert_eq!(Big::ENDIAN.endian(), Some(Endian::Big));
    assert_eq!(Little::ENDIAN.endian(), Some(Endian::Little));
    assert_eq!(u8::ENDIAN, EndianKind::None);
    assert_eq!(u8::ENDIAN.endian(), None);
    assert_eq!(Runtime::ENDIAN, EndianKind::Runtime);
    assert_eq!(Runtime::ENDIAN.endian(), None);
    assert_eq!(<(u8, Big)>::ENDIAN, EndianKind::Mixed);
    assert_eq!(<(u8, Big)>::ENDIAN.endian(), None);
}
