#[test]
fn null_wide_strings() {
    use binrw::{io::Cursor, BinReaderExt, NullWideString};

    const WIDE_STRINGS: &[u8] = b"w\0i\0d\0e\0 \0s\0t\0r\0i\0n\0g\0s\0\0\0";
    const ARE_ENDIAN_DEPENDENT: &[u8] =
        b"\0a\0r\0e\0 \0e\0n\0d\0i\0a\0n\0 \0d\0e\0p\0e\0n\0d\0e\0n\0t\0\0";

    let mut wide_strings = Cursor::new(WIDE_STRINGS);
    let mut are_endian_dependent = Cursor::new(ARE_ENDIAN_DEPENDENT);

    let wide_strings: NullWideString = wide_strings.read_le().unwrap();
    let are_endian_dependent: NullWideString = are_endian_dependent.read_be().unwrap();

    assert_eq!(
        // notice: read_le
        wide_strings.into_string(),
        "wide strings"
    );

    assert_eq!(
        // notice: read_be
        are_endian_dependent.into_string(),
        "are endian dependent"
    );
}

#[test]
fn null_strings() {
    use binrw::{io::Cursor, BinReaderExt, NullString};

    let mut null_separated_strings =
        Cursor::new(b"null terminated strings? in my system's language?\0no thanks\0");

    assert_eq!(
        null_separated_strings
            .read_be::<NullString>()
            .unwrap()
            .into_string(),
        "null terminated strings? in my system's language?"
    );

    assert_eq!(
        null_separated_strings
            .read_be::<NullString>()
            .unwrap()
            .into_string(),
        "no thanks"
    );
}

#[test]
fn null_string_round_trip() {
    use binrw::{io::Cursor, BinReaderExt, BinWriterExt, NullString};

    let data = "test test test";
    let s = NullString::from_string(String::from(data));

    let mut x = Cursor::new(Vec::new());
    x.write_be(&s).unwrap();

    let s2: NullString = Cursor::new(x.into_inner()).read_be().unwrap();

    assert_eq!(&s2.into_string(), data);
}

#[test]
fn null_wide_string_round_trip() {
    use binrw::{io::Cursor, BinReaderExt, BinWriterExt, NullWideString};

    let data = "test test test";
    let s = NullWideString::from_string(String::from(data));

    let mut x = Cursor::new(Vec::new());
    x.write_be(&s).unwrap();

    let s2: NullWideString = Cursor::new(x.into_inner()).read_be().unwrap();

    assert_eq!(&s2.into_string(), data);
}
