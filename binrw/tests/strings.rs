#[test]
fn null_wide_strings() {
    use binrw::{io::Cursor, BinReaderExt, NullWideString};

    assert_eq!(
        Cursor::new(b"w\0i\0d\0e\0 \0s\0t\0r\0i\0n\0g\0s\0\0\0")
            .read_le::<NullWideString>()
            .unwrap()
            .to_string(),
        "wide strings"
    );

    assert_eq!(
        Cursor::new(b"\0a\0r\0e\0 \0e\0n\0d\0i\0a\0n\0 \0d\0e\0p\0e\0n\0d\0e\0n\0t\0\0")
            .read_be::<NullWideString>()
            .unwrap()
            .to_string(),
        "are endian dependent"
    );

    assert_eq!(
        format!(
            "{:?}",
            Cursor::new(b"d\0e\0b\0u\0g\0\x3a\x26\n\0\0\0")
                .read_le::<NullWideString>()
                .unwrap()
        ),
        "NullWideString(\"debug☺\\n\")"
    );

    assert_eq!(
        format!(
            "{:?}",
            Cursor::new(b"b\0a\0d\0 \0\0\xdc\0\xdc \0s\0u\0r\0r\0o\0g\0a\0t\0e\0\0\0")
                .read_le::<NullWideString>()
                .unwrap()
        ),
        "NullWideString(\"bad \u{FFFD}\u{FFFD} surrogate\")"
    );

    // Default/Deref/DerefMut
    let mut s = NullWideString::default();
    s.extend_from_slice(&[b'h'.into(), b'e'.into(), b'y'.into()]);
    assert_eq!(&s[0..2], &[b'h'.into(), b'e'.into()]);

    // Clone/TryFrom
    let t = String::try_from(s.clone()).unwrap();
    assert_eq!(t, "hey");
    s.push(0xdc00);
    String::try_from(s).expect_err("accepted bad data");

    // From
    let s = NullWideString::from(t.clone());
    assert_eq!(Vec::from(s), t.encode_utf16().collect::<Vec<_>>());
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
            .to_string(),
        "null terminated strings? in my system's language?"
    );

    assert_eq!(
        null_separated_strings
            .read_be::<NullString>()
            .unwrap()
            .to_string(),
        "no thanks"
    );

    assert_eq!(
        format!(
            "{:?}",
            Cursor::new(b"debug\xe2\x98\xba\n\0")
                .read_be::<NullString>()
                .unwrap()
        ),
        "NullString(\"debug☺\\n\")"
    );

    assert_eq!(
        format!(
            "{:?}",
            Cursor::new(b"bad \xfe utf8 \xfe\0")
                .read_be::<NullString>()
                .unwrap()
        ),
        "NullString(\"bad \u{FFFD} utf8 \u{FFFD}\")"
    );

    assert_eq!(
        format!(
            "{:?}",
            Cursor::new(b"truncated\xe2\0")
                .read_be::<NullString>()
                .unwrap()
        ),
        "NullString(\"truncated\u{FFFD}\")"
    );

    // Default/Deref/DerefMut
    let mut s = NullString::default();
    s.extend_from_slice(b"hey");
    assert_eq!(&s[0..2], b"he");

    // Clone/TryFrom
    let t = String::try_from(s.clone()).unwrap();
    assert_eq!(t, "hey");
    s.extend_from_slice(b"\xe2");
    String::try_from(s).expect_err("accepted bad data");

    // From
    let s = NullString::from(t.clone());
    assert_eq!(Vec::from(s), t.as_bytes());
}

#[test]
fn null_string_round_trip() {
    use binrw::{io::Cursor, BinReaderExt, BinWriterExt, NullString};

    let data = "test test test";
    let s = NullString::from(data);

    let mut x = Cursor::new(Vec::new());
    x.write_be(&s).unwrap();

    let s2: NullString = Cursor::new(x.into_inner()).read_be().unwrap();

    assert_eq!(&s2.to_string(), data);
}

#[test]
fn null_wide_string_round_trip() {
    use binrw::{io::Cursor, BinReaderExt, BinWriterExt, NullWideString};

    let data = "test test test";
    let s = NullWideString::from(data);

    let mut x = Cursor::new(Vec::new());
    x.write_be(&s).unwrap();

    let s2: NullWideString = Cursor::new(x.into_inner()).read_be().unwrap();

    assert_eq!(&s2.to_string(), data);
}
