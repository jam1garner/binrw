use binrw::{binrw, io::Cursor, BinReaderExt, BinResult, BinWriterExt};

#[binrw::parser]
fn single_arg_parser(arg: u32) -> BinResult<u32> {
    Ok(arg)
}

#[binrw::writer(writer)]
fn single_arg_writer(_object: &u32, arg: u32) -> BinResult<()> {
    writer.write_le(&arg)
}

#[binrw]
struct SingleArg {
    #[br(parse_with = single_arg_parser, args(0x42u32))]
    #[bw(write_with = single_arg_writer, args(0x42u32))]
    field: u32,
}

#[test]
fn single_arg() {
    let result: SingleArg = Cursor::new(b"").read_le().unwrap();
    assert_eq!(result.field, 0x42);
    let mut written = Cursor::new(Vec::new());
    written.write_le(&result).unwrap();
    assert_eq!(written.into_inner(), b"\x42\x00\x00\x00");
}
