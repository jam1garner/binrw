use binrw::{io::Cursor, BinWrite};

#[test]
fn derive_allows_default() {
    #[derive(BinWrite)]
    struct Test<T>
    where
        T: BinWrite,
        for<'a> T::Args<'a>: Default,
    {
        a: T,
    }

    let mut result = Vec::new();
    Test::<u8> { a: 0 }
        .write_be(&mut Cursor::new(&mut result))
        .unwrap();
    assert_eq!(b"\0", &result[..]);
}
