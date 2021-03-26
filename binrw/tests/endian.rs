use binrw::Endian;

#[test]
fn endian_to_string() {
    assert_eq!(Endian::Big.to_string(), "Big");
    assert_eq!(Endian::Little.to_string(), "Little");
    assert_eq!(Endian::Native.to_string(), "Native");
}
