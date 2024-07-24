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

#[test]
fn write_with_fn_once_closure_args() {
    #[derive(BinWrite)]
    #[bw(little)]
    struct Test {
        #[bw(args(1), write_with = |_, s, e, (a,): (u8,)| a.write_options(s, e, ()))]
        a: u8,
    }

    let mut x = Cursor::new(Vec::new());
    Test { a: 0 }.write(&mut x).unwrap();
    assert_eq!(x.into_inner(), b"\x01");
}
