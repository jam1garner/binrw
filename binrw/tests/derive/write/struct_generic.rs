use std::marker::PhantomData;

use binrw::{binwrite, io::Cursor, BinWrite};

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
fn derive_allows_calc_default() {
    #[binwrite]
    pub struct Test<T: BinWrite + Default>
    where
        for<'a> <T as BinWrite>::Args<'a>: Default,
    {
        #[bw(calc = T::default())]
        a: T,
        _phantom: PhantomData<T>,
    }

    let mut result = Vec::new();
    Test::<u8> {
        _phantom: PhantomData,
    }
    .write_be(&mut Cursor::new(&mut result))
    .unwrap();
    assert_eq!(b"\0", &result[..]);
}
