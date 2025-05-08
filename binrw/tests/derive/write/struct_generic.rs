use binrw::{io::Cursor, BinWrite};

#[test]
fn derive_allows_default() {
    #[derive(BinWrite)]
    struct Test<T>
    where
        T: BinWrite,
        for<'a> T::Args<'a>: Default,
    {
        a: u16,
        b: T,
    }

    let mut result = Vec::new();
    Test::<u8> { a: 0, b: 1 }
        .write_be(&mut Cursor::new(&mut result))
        .unwrap();
    assert_eq!(b"\0\0\x01", &result[..]);
}

#[test]
fn derive_generic_bound() {
    #[derive(BinWrite)]
    #[bw(bound(for<'a> T: BinWrite + 'a, for<'a> T::Args<'a>: Default + Clone))]
    struct Test<T> {
        a: [T; 3],
    }

    let mut result = Vec::new();
    Test::<u8> { a: [0, 1, 2] }
        .write_le(&mut Cursor::new(&mut result))
        .unwrap();
    assert_eq!(b"\0\x01\x02", &result[..]);
}
