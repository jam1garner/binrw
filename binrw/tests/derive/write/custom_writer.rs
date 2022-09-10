use binrw::{io::Cursor, BinWrite, Endian, WriteOptions};

#[test]
fn custom_writer() {
    #[derive(BinWrite)]
    struct Test {
        x: u8,

        #[bw(write_with = custom_writer)]
        y: u16,
    }

    fn custom_writer<W: binrw::io::Write + binrw::io::Seek>(
        _this: &u16,
        writer: &mut W,
        _opts: &WriteOptions,
        _: (),
    ) -> binrw::BinResult<()> {
        writer.write_all(b"abcd")?;
        Ok(())
    }

    let mut x = Cursor::new(Vec::new());

    Test { x: 1, y: 2 }
        .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    assert_eq!(x.into_inner(), b"\x01abcd");
}
