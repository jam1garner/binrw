use binrw::Endian;

#[test]
fn endian_bom() {
    assert_eq!(
        Endian::from_utf16_bom_bytes([0xfe, 0xff]).unwrap(),
        Endian::Big
    );
    assert_eq!(
        Endian::from_utf16_bom_bytes([0xff, 0xfe]).unwrap(),
        Endian::Little
    );
    assert!(Endian::from_utf16_bom_bytes([0xfa, 0xff]).is_err());
    assert_eq!(Endian::Little.into_utf16_bom_bytes(), [0xff, 0xfe]);
    assert_eq!(Endian::Big.into_utf16_bom_bytes(), [0xfe, 0xff]);
}

#[test]
fn endian_to_string() {
    assert_eq!(Endian::Big.to_string(), "Big");
    assert_eq!(Endian::Little.to_string(), "Little");
}
