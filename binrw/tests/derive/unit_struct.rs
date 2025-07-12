extern crate binrw;
use super::t;

#[test]
fn unit_struct_magic() {
    #[derive(binrw::BinRead, Debug)]
    #[br(big, magic = 1u16)]
    struct Test;

    <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x00\x01")).unwrap();
    let error = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x00\x00"))
        .expect_err("accepted bad data");
    assert!(t::matches!(error, binrw::Error::BadMagic { .. }));
}

#[test]
fn unit_struct_import_pre_assert() {
    #[derive(binrw::BinRead, Debug)]
    #[br(import { succeed: bool }, pre_assert(succeed))]
    struct Test;

    <Test as binrw::BinRead>::read_args(
        &mut binrw::io::Cursor::new(b""),
        binrw::args! { succeed: true },
    )
    .unwrap();
    let error = <Test as binrw::BinRead>::read_args(
        &mut binrw::io::Cursor::new(b""),
        binrw::args! { succeed: false },
    )
    .expect_err("accepted negative pre-assert");
    assert!(t::matches!(error, binrw::Error::AssertFail { .. }));
}
