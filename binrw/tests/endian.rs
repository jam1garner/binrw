use binrw::Endian;

#[test]
fn endian_to_string() {
    assert_eq!(String::from(&Endian::Big), "Big");
    assert_eq!(String::from(&Endian::Little), "Little");
    assert_eq!(String::from(&Endian::Native), "Native");
}
