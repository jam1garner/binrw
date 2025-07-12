extern crate binrw;
use super::t;

#[test]
fn custom_writer() {
    #[derive(binrw::BinWrite)]
    struct Test {
        x: u8,

        #[bw(write_with = custom_writer)]
        y: u16,
    }

    #[binrw::writer(writer)]
    fn custom_writer(_this: &u16) -> binrw::BinResult<()> {
        writer.write_all(b"abcd")?;
        t::Ok(())
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());

    binrw::BinWrite::write_options(&Test { x: 1, y: 2 }, &mut x, binrw::Endian::Big, ()).unwrap();

    t::assert_eq!(x.into_inner(), b"\x01abcd");
}

#[test]
fn write_with_fn_once_closure_args() {
    #[derive(binrw::BinWrite)]
    #[bw(little)]
    struct Test {
        #[bw(args(1), write_with = |_, s, e, (a,): (u8,)| a.write_options(s, e, ()))]
        a: u8,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write(&Test { a: 0 }, &mut x).unwrap();
    t::assert_eq!(x.into_inner(), b"\x01");
}

#[binrw::writer(writer)]
fn write_as_ref_str<S: t::AsRef<str>>(value: S) -> binrw::BinResult<()> {
    let bytes = value.as_ref().as_bytes();
    writer.write_all(bytes)?;
    t::Ok(())
}

#[test]
fn write_with_as_ref_str() {
    use binrw::prelude::*;

    #[derive(binrw::BinWrite)]
    struct MyType {
        #[bw(write_with = write_as_ref_str)]
        value: t::String,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    MyType {
        value: t::ToString::to_string("Hello, World!"),
    }
    .write_le(&mut x)
    .unwrap();
    t::assert_eq!(x.into_inner(), b"Hello, World!");
}

#[test]
fn map_write_with_as_ref_str() {
    use binrw::prelude::*;

    #[derive(BinWrite)]
    struct MyType {
        #[bw(map = t::ToString::to_string, write_with = write_as_ref_str)]
        value: u32,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    MyType { value: 42 }.write_le(&mut x).unwrap();
    t::assert_eq!(x.into_inner(), b"42");
}

#[test]
fn try_map_write_with_as_ref_str() {
    use binrw::prelude::*;

    #[derive(binrw::BinWrite)]
    struct MyType<'a> {
        #[bw(try_map = |x| x.ok_or("Option was None"), write_with = write_as_ref_str)]
        value: t::Option<&'a str>,
    }

    let mut x = binrw::io::Cursor::new(t::Vec::new());
    MyType {
        value: t::Some("Hello, World!"),
    }
    .write_le(&mut x)
    .unwrap();
    t::assert_eq!(x.into_inner(), b"Hello, World!");
}
