extern crate binrw;

#[binrw::parser]
fn single_arg_parser(arg: u32) -> binrw::BinResult<u32> {
    binrw::BinResult::Ok(arg)
}

#[binrw::writer(writer)]
fn single_arg_writer(_object: &u32, arg: u32) -> binrw::BinResult<()> {
    binrw::BinWrite::write_le(&arg, writer)
}

#[binrw::binrw]
struct SingleArg {
    #[br(parse_with = single_arg_parser, args(0x42u32))]
    #[bw(write_with = single_arg_writer, args(0x42u32))]
    field: u32,
}

#[test]
fn single_arg() {
    use super::t::*;
    use binrw::{io::Cursor, BinRead, BinWrite};

    let result = SingleArg::read_le(&mut Cursor::new(b"")).unwrap();
    assert_eq!(result.field, 0x42);
    let mut written = Cursor::new(Vec::new());
    result.write_le(&mut written).unwrap();
    assert_eq!(written.into_inner(), b"\x42\x00\x00\x00");
}
