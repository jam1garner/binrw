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

#[binrw::writer(writer)]
fn write_as_ref_str<S: AsRef<str>>(value: S) -> binrw::BinResult<()> {
    let bytes = value.as_ref().as_bytes();
    writer.write_all(bytes)?;
    Ok(())
}

#[test]
fn write_with_as_ref_str() {
    use binrw::prelude::*;

    #[derive(BinWrite)]
    struct MyType {
        #[bw(write_with = write_as_ref_str)]
        value: String,
    }

    let mut x = Cursor::new(Vec::new());
    MyType {
        value: "Hello, World!".to_string(),
    }
    .write_le(&mut x)
    .unwrap();
    assert_eq!(x.into_inner(), b"Hello, World!");
}

#[test]
fn map_write_with_as_ref_str() {
    use binrw::prelude::*;

    #[derive(BinWrite)]
    struct MyType {
        #[bw(map = |x| x.to_string(), write_with = write_as_ref_str)]
        value: u32,
    }

    let mut x = Cursor::new(Vec::new());
    MyType { value: 42 }.write_le(&mut x).unwrap();
    assert_eq!(x.into_inner(), b"42");
}
