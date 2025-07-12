extern crate binrw;
use super::t;

#[test]
fn derive_generic() {
    #[derive(binrw::BinRead)]
    struct Test<T: for<'a> binrw::BinRead<Args<'a> = ()> + t::Default> {
        a: [T; 3],
    }

    let result =
        <Test<u8> as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\0\x01\x02")).unwrap();
    t::assert_eq!(result.a, [0, 1, 2]);
}
