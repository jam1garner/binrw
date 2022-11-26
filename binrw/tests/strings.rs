use binrw::{io::Cursor, BinReaderExt, NullString, NullWideString, ReadWith};

#[test]
fn null_strings() {
    use binrw::WriteWith;

    let mut null_separated_strings =
        Cursor::new(b"null terminated strings? in my system's language?\0no thanks\0");

    assert_eq!(
        null_separated_strings
            .read_with::<NullString, String>()
            .unwrap(),
        "null terminated strings? in my system's language?"
    );

    assert_eq!(
        null_separated_strings
            .read_with::<NullString, String>()
            .unwrap(),
        "no thanks"
    );

    Cursor::new(b"no terminator")
        .read_with::<NullString, Vec<u8>>()
        .unwrap_err();

    Cursor::new(b"bad utf8\xc3\x28\0")
        .read_with::<NullString, String>()
        .unwrap_err();

    assert_eq!(
        String::read_with::<NullString, _>(&mut Cursor::new(b"test\0")).unwrap(),
        "test"
    );

    let s = String::from("test");
    let mut out = Cursor::new(Vec::new());
    s.write_with::<NullString, _>(&mut out).unwrap();
    assert_eq!(out.into_inner(), b"test\0");
}

#[test]
fn null_wide_strings() {
    use binrw::WriteWith;

    assert_eq!(
        Cursor::new(b"w\0i\0d\0e\0 \0s\0t\0r\0i\0n\0g\0s\0\0\0")
            .read_le_with::<NullWideString, String>()
            .unwrap(),
        "wide strings"
    );

    assert_eq!(
        Cursor::new(b"\0a\0r\0e\0 \0e\0n\0d\0i\0a\0n\0 \0d\0e\0p\0e\0n\0d\0e\0n\0t\0\0")
            .read_be_with::<NullWideString, String>()
            .unwrap(),
        "are endian dependent"
    );

    Cursor::new(b"bad utf16\0\xd8\x3d\0\x27\0\0")
        .read_be_with::<NullWideString, String>()
        .unwrap_err();

    Cursor::new(b"\0n\0o\0t\0e\0r\0m")
        .read_be_with::<NullWideString, Vec<u16>>()
        .unwrap_err();

    assert_eq!(
        String::read_be_with::<NullWideString, _>(&mut Cursor::new(b"\0t\0e\0s\0t\0\0")).unwrap(),
        "test"
    );
    assert_eq!(
        String::read_le_with::<NullWideString, _>(&mut Cursor::new(b"t\0e\0s\0t\0\0\0")).unwrap(),
        "test"
    );
    assert_eq!(
        String::read_ne_with::<NullWideString, _>(&mut Cursor::new(
            if cfg!(target_endian = "big") {
                b"\0t\0e\0s\0t\0\0"
            } else {
                b"t\0e\0s\0t\0\0\0"
            }
        ))
        .unwrap(),
        "test"
    );
    assert_eq!(
        Cursor::new(if cfg!(target_endian = "big") {
            b"\0t\0e\0s\0t\0\0"
        } else {
            b"t\0e\0s\0t\0\0\0"
        })
        .read_ne_with::<NullWideString, String>()
        .unwrap(),
        "test"
    );

    let s = String::from("test");
    let mut out = Cursor::new(Vec::new());
    s.write_be_with::<NullWideString, _>(&mut out).unwrap();
    assert_eq!(out.into_inner(), b"\0t\0e\0s\0t\0\0");
    let mut out = Cursor::new(Vec::new());
    s.write_le_with::<NullWideString, _>(&mut out).unwrap();
    assert_eq!(out.into_inner(), b"t\0e\0s\0t\0\0\0");
    let mut out = Cursor::new(Vec::new());
    s.write_ne_with::<NullWideString, _>(&mut out).unwrap();
    assert_eq!(
        out.into_inner(),
        if cfg!(target_endian = "big") {
            b"\0t\0e\0s\0t\0\0"
        } else {
            b"t\0e\0s\0t\0\0\0"
        }
    );
}

#[test]
fn bin_writer_ext() {
    use binrw::BinWriterExt;

    let s = String::from("test");
    let mut out = Cursor::new(Vec::new());
    out.write_be_with::<NullWideString, _>(&s).unwrap();
    assert_eq!(out.into_inner(), b"\0t\0e\0s\0t\0\0");
    let mut out = Cursor::new(Vec::new());
    out.write_le_with::<NullWideString, _>(&s).unwrap();
    assert_eq!(out.into_inner(), b"t\0e\0s\0t\0\0\0");
    let mut out = Cursor::new(Vec::new());
    out.write_ne_with::<NullWideString, _>(&s).unwrap();
    assert_eq!(
        out.into_inner(),
        if cfg!(target_endian = "big") {
            b"\0t\0e\0s\0t\0\0"
        } else {
            b"t\0e\0s\0t\0\0\0"
        }
    );
}

#[test]
fn null_string_round_trip() {
    use binrw::BinWriterExt;

    // str
    let s = "test test test";
    let mut x = Cursor::new(Vec::new());
    x.write_with::<NullString, _>(s).unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_with::<NullString, String>()
            .unwrap(),
        s
    );

    // String
    let mut x = Cursor::new(Vec::new());
    x.write_with::<NullString, _>(&s.to_string()).unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_with::<NullString, String>()
            .unwrap(),
        s
    );

    // [u8; N]
    let s = b"test test test";
    let mut x = Cursor::new(Vec::new());
    x.write_with::<NullString, _>(s).unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_with::<NullString, Vec<u8>>()
            .unwrap(),
        s
    );

    // [u8]
    let mut x = Cursor::new(Vec::new());
    x.write_with::<NullString, _>(s.as_slice()).unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_with::<NullString, Vec<u8>>()
            .unwrap(),
        s
    );

    // Vec<u8>
    let mut x = Cursor::new(Vec::new());
    x.write_with::<NullString, _>(&s.to_vec()).unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_with::<NullString, Vec<u8>>()
            .unwrap(),
        s
    );
}

#[test]
fn null_wide_string_round_trip() {
    use binrw::BinWriterExt;

    // str
    let s = "test test test";
    let mut x = Cursor::new(Vec::new());
    x.write_be_with::<NullWideString, _>(s).unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_be_with::<NullWideString, String>()
            .unwrap(),
        s
    );

    // String
    let mut x = Cursor::new(Vec::new());
    x.write_be_with::<NullWideString, _>(&s.to_string())
        .unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_be_with::<NullWideString, String>()
            .unwrap(),
        s
    );

    // [u16; N]
    let s = b"test test test".map(u16::from);
    let mut x = Cursor::new(Vec::new());
    x.write_be_with::<NullWideString, _>(&s).unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_be_with::<NullWideString, Vec<u16>>()
            .unwrap(),
        s
    );

    // [u16]
    let mut x = Cursor::new(Vec::new());
    x.write_be_with::<NullWideString, _>(s.as_slice()).unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_be_with::<NullWideString, Vec<u16>>()
            .unwrap(),
        s
    );

    // Vec<u16>
    let mut x = Cursor::new(Vec::new());
    x.write_be_with::<NullWideString, _>(&s.to_vec()).unwrap();
    assert_eq!(
        Cursor::new(x.into_inner())
            .read_be_with::<NullWideString, Vec<u16>>()
            .unwrap(),
        s
    );
}
