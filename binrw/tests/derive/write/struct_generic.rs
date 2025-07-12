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
