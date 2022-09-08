use binrw::{io::Cursor, BinWrite, Endian};

#[test]
fn custom_writer() {
    #[derive(BinWrite)]
    struct Test {
        x: u8,

        #[bw(write_with = custom_writer)]
        y: u16,
    }

    #[binrw::writer(writer)]
    fn custom_writer(_this: &u16) -> binrw::BinResult<()> {
        writer.write_all(b"abcd")?;
        Ok(())
    }

    let mut x = Cursor::new(Vec::new());

    Test { x: 1, y: 2 }
        .write_options(&mut x, Endian::Big, ())
        .unwrap();

    assert_eq!(x.into_inner(), b"\x01abcd");
}
