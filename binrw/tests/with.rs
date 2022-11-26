use binrw::{io::Cursor, BinRead, BinWrite, NullString, With};

#[test]
fn with() {
    let mut s = With::<NullString, String>::read(&mut Cursor::new(b"test\0")).unwrap();
    assert_eq!(*s, "test");
    *s = "mutable".to_string();
    assert_eq!(format!("{:?}", s), "With(\"mutable\")");
    let s2 = s.clone();
    assert_eq!(s, s2);
    assert_eq!(s2.into_inner(), "mutable");
    let mut out = Cursor::new(Vec::new());
    s.write(&mut out).unwrap();
    assert_eq!(out.into_inner(), b"mutable\0");
    let a = With::<NullString, &'static str>::from("a");
    let b = With::<NullString, &'static str>::from("b");
    assert_eq!(a.partial_cmp(&b), Some(core::cmp::Ordering::Less));
    assert_eq!(a.cmp(&b), core::cmp::Ordering::Less);
}
