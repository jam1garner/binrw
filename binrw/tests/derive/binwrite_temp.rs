extern crate binrw;
use super::t;

#[test]
fn binwrite_temp_applies() {
    #[binrw::binrw]
    #[derive(Debug, PartialEq)]
    #[br(big)]
    struct Test {
        #[bw(calc = vec.len() as u32)]
        len: u32,

        #[br(count = len)]
        vec: t::Vec<u8>,
    }

    let result =
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\0\0\x05ABCDE")).unwrap();
    // This also indirectly checks that `temp` is actually working since
    // compilation would fail if it weren’t due to missing the `len` field
    t::assert_eq!(
        result,
        Test {
            vec: b"ABCDE".to_vec()
        }
    );
}

#[test]
fn binwrite_temp_with_ignore() {
    #[binrw::binrw]
    #[derive(Debug, PartialEq)]
    #[br(big)]
    struct Test {
        #[br(temp)]
        #[bw(ignore)]
        len: u32,

        #[br(count = len)]
        vec: t::Vec<u8>,
    }

    let result =
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\0\0\x05ABCDE")).unwrap();
    t::assert_eq!(
        result,
        Test {
            vec: b"ABCDE".to_vec()
        }
    );

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(&result, &mut x, binrw::Endian::Big, ()).unwrap();

    // Since it's bw(ignore), the length isn't written here
    t::assert_eq!(x.into_inner(), b"ABCDE");
}
