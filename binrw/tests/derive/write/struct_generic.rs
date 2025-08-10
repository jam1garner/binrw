extern crate binrw;
use super::t;

#[test]
fn derive_allows_default() {
    #[derive(binrw::BinWrite)]
    struct Test<T>
    where
        T: binrw::BinWrite,
        for<'a> T::Args<'a>: t::Default,
    {
        a: u16,
        b: T,
    }

    let mut result = t::Vec::new();
    binrw::BinWrite::write_be(
        &Test::<u8> { a: 0, b: 1 },
        &mut binrw::io::Cursor::new(&mut result),
    )
    .unwrap();
    t::assert_eq!(b"\0\0\x01", &result[..]);
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
