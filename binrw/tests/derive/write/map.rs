use binrw::{binwrite, io::Cursor, BinWrite};

#[test]
fn map_field() {
    #[binwrite]
    #[bw(big)]
    struct Test {
        #[bw(map = |&x| x as u64)]
        x: u32,

        #[bw(map = |x| x.as_bytes())]
        y: String,

        #[bw(calc = 0xff, map = |x: u8| x)]
        z: u8,
    }

    let mut x = Cursor::new(Vec::new());

    Test {
        x: 3,
        y: String::from("test"),
    }
    .write(&mut x)
    .unwrap();

    assert_eq!(x.into_inner(), b"\0\0\0\0\0\0\0\x03test\xff");
}

#[test]
fn map_field_code_coverage() {
    #[allow(dead_code)]
    #[derive(BinWrite)]
    struct Test {
        #[bw(map = |&x| x as u64)]
        x: u32,

        #[bw(map = |x| x.as_bytes())]
        y: String,
    }
}

#[test]
fn map_repr_enum() {
    #[allow(dead_code)]
    #[derive(BinWrite, Debug)]
    #[bw(repr = u8)]
    enum Test {
        SubTest(u8),
    }

    impl From<&Test> for u8 {
        fn from(t: &Test) -> Self {
            match t {
                Test::SubTest(u) => *u,
            }
        }
    }
}

#[test]
fn map_repr_enum_variant() {
    #[allow(dead_code)]
    #[derive(BinWrite, Debug)]
    enum Test {
        SubTest(#[bw(repr = u8)] SubTest),
    }

    #[derive(Debug)]
    struct SubTest(u8);

    impl From<&SubTest> for u8 {
        fn from(s: &SubTest) -> Self {
            s.0
        }
    }
}

#[test]
fn map_repr_struct() {
    #[derive(BinWrite, Debug)]
    #[bw(repr = u8)]
    struct Test {
        a: u8,
    }

    impl From<&Test> for u8 {
        fn from(t: &Test) -> Self {
            t.a
        }
    }
}

#[test]
fn map_repr_struct_field() {
    #[allow(dead_code)]
    #[derive(BinWrite, Debug)]
    #[bw(big)]
    struct Test {
        #[bw(repr = u8)]
        a: SubTest,
    }

    #[derive(Debug)]
    struct SubTest {
        a: u8,
    }

    impl From<&SubTest> for u8 {
        fn from(s: &SubTest) -> Self {
            s.a
        }
    }
}

#[test]
fn try_map() {
    use binrw::prelude::*;

    #[derive(BinWrite)]
    struct MyType {
        #[bw(try_map = |&x| { i8::try_from(x) })]
        value: u8,
    }

    let mut x = Cursor::new(Vec::new());
    MyType { value: 127 }.write_le(&mut x).unwrap();
    assert_eq!(x.into_inner(), b"\x7f");

    let mut x = Cursor::new(Vec::new());
    MyType { value: 128 }.write_le(&mut x).unwrap_err();
}

#[test]
fn map_write_with() {
    use binrw::prelude::*;

    #[derive(BinWrite)]
    struct MyType {
        #[bw(map = |&x| x as u16, write_with = <u16 as BinWrite>::write_options)]
        value: u8,
    }

    let mut x = Cursor::new(Vec::new());
    MyType { value: 127 }.write_le(&mut x).unwrap();
    assert_eq!(x.into_inner(), b"\x7f\0");
}
