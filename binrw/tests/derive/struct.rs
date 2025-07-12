extern crate binrw;
use super::t;

#[test]
fn all_the_things() {
    #[derive(Debug)]
    struct PlainObject;

    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    #[br(is_big = true, magic = b"TEST")]
    #[br(assert(entries.len() as u32 == extra_entry_count + 1))]
    struct Test {
        extra_entry_count: u32,

        #[br(count = extra_entry_count + 1, args { inner: binrw::args! { inner: binrw::args! { extra_val: 0x69 } } })]
        entries: t::Vec<binrw::file_ptr::FilePtr<u32, TestEntry>>,

        #[br(default)]
        start_as_none: t::Option<PlainObject>,

        #[br(calc = 1 + 2)]
        calc_test: u32,
    }

    #[binrw::parser(reader, endian)]
    fn read_offsets() -> binrw::BinResult<(u16, u16)> {
        t::Ok((
            <u16 as binrw::BinRead>::read_options(reader, endian, ())?,
            <u16 as binrw::BinRead>::read_options(reader, endian, ())?,
        ))
    }

    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    #[br(little, magic = b"TST2")]
    #[br(import { extra_val: u8 })]
    struct TestEntry {
        #[br(map = |val: u32| t::ToString::to_string(&val))]
        entry_num: t::String,

        #[br(assert(offsets.1 - offsets.0 == 0x10))]
        #[br(seek_before(binrw::io::SeekFrom::Current(4)))]
        #[br(parse_with = read_offsets)]
        #[br(is_big = entry_num == "1")]
        offsets: (u16, u16),

        #[br(if(offsets.0 == 0x20))]
        name: t::Option<binrw::file_ptr::FilePtr<u32, binrw::strings::NullString>>,

        #[br(calc(extra_val))]
        extra_val: u8,
    }

    <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(include_bytes!(
        "data/test_file.bin"
    )))
    .unwrap();
}

#[test]
fn assert() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    struct Test {
        #[br(assert(a == 1))]
        a: u8,
    }

    <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new("\x01")).unwrap();
    let error = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new("\0"))
        .expect_err("accepted bad data");
    match error {
        binrw::Error::AssertFail { pos, message } => {
            t::assert_eq!(pos, 0);
            t::assert_eq!(message, "assertion failed: `a == 1`");
        }
        _ => t::panic!("bad error type"),
    }
}

#[test]
fn assert_custom_err() {
    #[derive(Debug)]
    struct Oops(u8);
    impl ::core::fmt::Display for Oops {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            t::write!(f, "oops!")
        }
    }

    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    struct Test {
        #[br(assert(a == 1, Oops(a)))]
        a: u8,
    }

    <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new("\x01")).unwrap();
    let error = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new("\x02"))
        .expect_err("accepted bad data");
    t::assert_eq!(t::format!("{error}"), "oops! at 0x0");
    let error = error.custom_err::<Oops>().expect("bad error type");
    t::assert_eq!(error.0, 2);
}

#[test]
fn assert_formatted() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    struct Test {
        #[br(assert(a == 1, "a was {}", a))]
        a: u8,
    }

    <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new("\x01")).unwrap();
    let error = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new("\0"))
        .expect_err("accepted bad data");
    match error {
        binrw::Error::AssertFail { pos, message } => {
            t::assert_eq!(pos, 0);
            t::assert_eq!(message, "a was 0");
        }
        _ => t::panic!("bad error type"),
    }
}

#[test]
fn calc_temp_field() {
    #[binrw::binread]
    #[derive(Debug, PartialEq)]
    #[br(big)]
    struct Test {
        #[br(temp)]
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
fn count_too_big() {
    #[derive(binrw::BinRead, Debug)]
    #[br(little)]
    struct Test {
        _a: u128,
        #[br(count = _a)]
        _b: t::Vec<u8>,
    }

    let error =
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(u128::MAX.to_le_bytes()))
            .expect_err("accepted bad count");
    match error {
        binrw::Error::AssertFail { pos, message } => {
            t::assert_eq!(pos, ::core::mem::size_of::<u128>() as u64);
            t::assert_eq!(
                message,
                t::format!("count {} out of range of usize", u128::MAX)
            );
        }
        _ => t::panic!("bad error type"),
    }
}

#[test]
fn count_no_useless_conversion_lint() {
    const LEN: usize = 1;
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(little)]
    struct Test {
        #[br(count = LEN)]
        data: t::Vec<u8>,
    }
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new("\x01")).unwrap(),
        Test {
            data: t::vec![1; 1]
        }
    );
}

#[test]
fn deref_now() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(big, magic = b"TEST")]
    struct Test {
        a: binrw::file_ptr::FilePtr<u32, binrw::strings::NullString>,
        b: i32,
    }

    let result = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(include_bytes!(
        "data/deref_now.bin"
    )))
    .unwrap();
    t::assert_eq!(
        result,
        Test {
            a: binrw::file_ptr::FilePtr {
                ptr: 0x10,
                value: binrw::strings::NullString(b"Test string".to_vec())
            },
            b: -1,
        }
    );
}

#[test]
fn move_args() {
    #[derive(Debug, PartialEq)]
    struct NonCopyArg;

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import(v: NonCopyArg))]
    struct Inner(#[br(calc = v)] NonCopyArg);

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import(v: NonCopyArg))]
    struct Test {
        #[br(args(v))]
        inner: Inner,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le_args(&mut binrw::io::Cursor::new(b""), (NonCopyArg,))
            .unwrap(),
        Test {
            inner: Inner(NonCopyArg)
        }
    );
}

#[test]
fn move_parser() {
    struct A;
    impl A {
        fn accept(&self, x: u32) -> bool {
            x == 0
        }
    }

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import(a: A))]
    struct Test {
        #[br(parse_with = binrw::helpers::until(|x| a.accept(*x)))]
        using_until: t::Vec<u32>,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le_args(
            &mut binrw::io::Cursor::new(b"\x01\0\0\0\x02\0\0\0\0\0\0\0"),
            (A,)
        )
        .unwrap(),
        Test {
            using_until: t::vec![1, 2, 0],
        }
    );
}

#[test]
fn move_stream() {
    #[binrw::binread]
    #[derive(Debug, PartialEq)]
    struct Test {
        #[br(map_stream = |r| r)]
        flags: u32,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x01\0\0\0")).unwrap(),
        Test { flags: 1 }
    );
}

#[test]
fn move_stream_with_count() {
    #[binrw::binread]
    #[derive(Debug, PartialEq)]
    struct Test {
        #[br(count = 1, map_stream = |r| r)]
        flags: t::Vec<u32>,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x01\0\0\0")).unwrap(),
        Test { flags: t::vec![1] }
    );
}

#[test]
fn move_named_stream_with_count() {
    #[binrw::binread]
    #[derive(Debug, PartialEq)]
    #[br(stream = s, map_stream = |r| r)]
    struct Test {
        #[br(count = 1)]
        flags: t::Vec<u32>,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x01\0\0\0")).unwrap(),
        Test { flags: t::vec![1] }
    );
}

// See https://github.com/jam1garner/binrw/issues/118
#[test]
fn move_temp_field() {
    #[binrw::binread]
    #[derive(Debug, Eq, PartialEq)]
    struct Foo {
        #[br(temp)]
        foo: binrw::NullString,

        #[br(calc = foo)]
        bar: binrw::NullString,
    }

    t::assert_eq!(
        <Foo as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"hello\0goodbyte\0"))
            .unwrap(),
        Foo {
            bar: <binrw::NullString as t::From<_>>::from("hello"),
        }
    );
}

#[test]
fn mut_map() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import(v: &mut t::Vec<u8>))]
    struct Test {
        #[br(map = |a: u8| a + v.pop().unwrap())]
        a: u8,
        #[br(map = |b: u8| b + v.pop().unwrap())]
        b: u8,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le_args(
            &mut binrw::io::Cursor::new(b"\x01\x02"),
            (&mut t::vec![1, 2],)
        )
        .unwrap(),
        Test { a: 3, b: 3 }
    );
}

#[test]
fn empty_imports() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import())]
    struct Test {
        a: u8,
    }

    let result = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x01")).unwrap();
    t::assert_eq!(result, Test { a: 1 });
}

#[test]
fn empty_named_imports() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import{})]
    struct Test {
        a: u8,
    }

    let result = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x01")).unwrap();
    t::assert_eq!(result, Test { a: 1 });
}

#[test]
fn all_default_imports() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import { _default: u8 = 42 })]
    struct Test {
        a: u8,
    }

    let result = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x01")).unwrap();
    t::assert_eq!(result, Test { a: 1 });
}

#[test]
fn recursive_lifetime_imports() {
    #[derive(Default)]
    struct InnerArgs<'a> {
        inner: &'a str,
    }
    #[derive(Default)]
    struct OuterArgs<'a, Extra> {
        outer: &'a str,
        extra: Extra,
    }
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import_raw(args: OuterArgs<'_, &'_ InnerArgs<'_>>))]
    struct Test {
        #[br(calc(<t::String as t::From<_>>::from(args.outer) + args.extra.inner))]
        a: t::String,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le_args(
            &mut binrw::io::Cursor::new(""),
            OuterArgs {
                outer: "hello",
                extra: &InnerArgs { inner: " world" }
            }
        )
        .unwrap(),
        Test {
            a: <t::String as t::From<_>>::from("hello world")
        }
    );
}

#[test]
fn gat_list() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(little, import(borrowed: &u8))]
    struct Test {
        #[br(calc(*borrowed))]
        a: u8,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b""), (&1_u8,)).unwrap(),
        Test { a: 1 }
    );
}

#[test]
fn gat_named() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(little, import { borrowed: &u8 })]
    struct Test {
        #[br(calc(*borrowed))]
        a: u8,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(
            &mut binrw::io::Cursor::new(b""),
            binrw::args! { borrowed: &1_u8 }
        )
        .unwrap(),
        Test { a: 1 }
    );
}

#[test]
fn gat_raw() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(little, import_raw(borrowed: &u8))]
    struct Test {
        #[br(calc(*borrowed))]
        a: u8,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b""), &1_u8).unwrap(),
        Test { a: 1 }
    );
}

#[test]
fn gat_map() {
    #[derive(binrw::BinRead)]
    #[br(import(borrowed: &u8))]
    struct Wrapper(#[br(calc = *borrowed)] u8);

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(little, import(borrowed: &u8))]
    struct Test {
        #[br(map = |x: Wrapper| x.0, args(borrowed))]
        a: u8,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b""), (&1_u8,)).unwrap(),
        Test { a: 1 }
    );
}

#[test]
fn if_alternate() {
    #[derive(binrw::BinRead, Debug)]
    #[br(import{ try_read: bool })]
    struct Test {
        #[br(if(try_read, 10))]
        a: u8,
    }

    let result = <Test as binrw::BinRead>::read_le_args(
        &mut binrw::io::Cursor::new(b"\x01"),
        <Test as binrw::BinRead>::Args::builder()
            .try_read(true)
            .finalize(),
    )
    .unwrap();
    t::assert_eq!(result.a, 1);
    let result = <Test as binrw::BinRead>::read_le_args(
        &mut binrw::io::Cursor::new(b"\x01"),
        binrw::args! { try_read: false },
    )
    .unwrap();
    t::assert_eq!(result.a, 10);
}

#[test]
fn ignore_and_default() {
    #[derive(Debug, Eq, PartialEq)]
    struct One(u8);
    impl t::Default for One {
        fn default() -> Self {
            Self(1)
        }
    }

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(big)]
    struct Test {
        a: u8,
        #[br(default)]
        b: One,
        #[br(ignore)]
        c: One,
    }

    let result = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x02")).unwrap();
    t::assert_eq!(
        result,
        Test {
            a: 2,
            b: t::Default::default(),
            c: t::Default::default()
        }
    );
}

#[test]
fn magic_byte() {
    #[derive(binrw::BinRead, Debug)]
    #[br(magic = b'a')]
    struct Test;

    <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"a")).unwrap();
    <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b""))
        .expect_err("accepted bad data");
    <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"x"))
        .expect_err("accepted bad data");
}

#[test]
fn magic_field() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(magic(b"A"))]
    struct Test {
        b: u8,
        #[br(magic(b"C"))]
        d: u8,
    }

    <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"ABBB"))
        .expect_err("accepted bad data");
    let result = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"ABCD")).unwrap();
    t::assert_eq!(result, Test { b: b'B', d: b'D' });
}

#[test]
fn magic_const() {
    use binrw::meta::ReadMagic;
    #[derive(binrw::BinRead, Debug)]
    #[br(magic = b'a')]
    struct Test;

    t::assert_eq!(Test::MAGIC, b'a');
}

#[test]
fn map_stream() {
    use binrw::io::TakeSeekExt;

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(magic = b"magic", map_stream = |reader| reader.take_seek(4))]
    struct Test {
        #[br(parse_with = binrw::helpers::until_eof)]
        a: t::Vec<u8>,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"magichello world"))
            .unwrap(),
        Test {
            a: b"hell".to_vec()
        }
    );
}

#[test]
fn map_stream_field() {
    use binrw::io::TakeSeekExt;

    #[derive(binrw::BinRead, Debug, PartialEq)]
    struct Test {
        #[br(magic = b"magic", map_stream = |reader| reader.take_seek(5), parse_with = binrw::helpers::until_eof)]
        a: t::Vec<u8>,
        b: u8,
        #[br(magic = b"magic", map_stream = |reader| reader.take_seek(5), parse_with = binrw::helpers::until_eof)]
        c: t::Vec<u8>,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"magichello magicworldx"))
            .unwrap(),
        Test {
            a: b"hello".to_vec(),
            b: b' ',
            c: b"world".to_vec(),
        }
    );
}

#[test]
fn map_stream_parse_with_args() {
    use binrw::{helpers::until_eof, io::TakeSeekExt};
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import(extra: u8))]
    struct Inner(#[br(map = |v: u8| v + extra)] u8);

    #[derive(binrw::BinRead, Debug, PartialEq)]
    struct Test {
        a: u8,
        #[br(map_stream = |s| s.take_seek(4), parse_with = until_eof, args(a))]
        b: t::Vec<Inner>,
        c: u8,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x0a\x00\x01\x02\x03\x04"))
            .unwrap(),
        Test {
            a: 10,
            b: t::vec![Inner(10), Inner(11), Inner(12), Inner(13)],
            c: 4
        }
    );
}

#[test]
fn pad_after_before() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    struct Test {
        #[br(pad_after = 1, pad_before = 1)]
        a: u8,
        b: u8,
    }

    let result =
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\0\x01\0\x02")).unwrap();
    t::assert_eq!(result, Test { a: 1, b: 2 });
}

#[test]
fn pad_size_to() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    struct Test {
        #[br(pad_before = 1, pad_size_to = 2)]
        a: u8,
        b: u8,
    }

    let result =
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\0\x01\0\x02")).unwrap();
    t::assert_eq!(result, Test { a: 1, b: 2 });
}

#[test]
fn parse_with_default_args() {
    #[derive(Clone)]
    struct Args(u8);
    impl t::Default for Args {
        fn default() -> Self {
            Self(42)
        }
    }

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import { in_a: u8 })]
    struct InnerImport {
        #[br(calc(in_a))]
        a: u8,
        b: u8,
    }

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import_raw(args: Args))]
    struct InnerImportTuple {
        #[br(calc(args.0))]
        a: u8,
        b: u8,
    }

    #[derive(binrw::BinRead, Debug, PartialEq)]
    struct Test {
        #[br(args{ in_a: 0 })]
        #[br(parse_with = InnerImport::read_options)]
        inner: InnerImport,
        #[br(parse_with = InnerImportTuple::read_options)]
        inner_tuple: InnerImportTuple,
    }

    let result =
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x02\x04")).unwrap();
    t::assert_eq!(
        result,
        Test {
            inner: InnerImport { a: 0, b: 2 },
            inner_tuple: InnerImportTuple { a: 42, b: 4 }
        }
    );
}

#[test]
fn args_type_hint_borrowck() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(import(a: u8))]
    struct NeedsArgs(#[br(map = |x: u8| x + a)] u8);

    #[derive(binrw::BinRead, Debug, PartialEq)]
    struct Test {
        #[br(args(4), parse_with = binrw::helpers::until(|x| x == &NeedsArgs(4)))]
        a: t::Vec<NeedsArgs>,
    }

    let result =
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x01\x00\x02")).unwrap();
    t::assert_eq!(
        result,
        Test {
            a: t::vec![NeedsArgs(5), NeedsArgs(4)]
        }
    );
}

#[test]
fn args_same_name() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    #[br(import { y: u16, x: u8 })]
    struct Test {
        #[br(calc(x))]
        z: u8,

        #[br(calc(y))]
        z2: u16,
    }

    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    struct Test2 {
        #[br(calc(3))]
        x: u8,

        #[br(args { x, y: 3 })]
        y: Test,
    }

    let result = <Test2 as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"")).unwrap();
    t::assert_eq!(result.y.z, 3);
}

#[test]
fn import_tuple() {
    #[derive(binrw::BinRead, Debug)]
    struct Test {
        #[br(args_raw = (1, 2))]
        a: Child,
    }

    #[derive(binrw::BinRead, Debug)]
    #[br(import_raw(args: (u8, u8)))]
    struct Child {
        #[br(calc(args.0 + args.1))]
        a: u8,
    }

    let result = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"")).unwrap();
    t::assert_eq!(result.a.a, 3);
}

#[test]
fn mixed_attrs() {
    #[binrw::binread]
    #[binrw::binwrite]
    #[brw(big)]
    struct Foo {
        a: Bar,
    }

    #[binrw::binwrite]
    #[binrw::binread]
    struct Bar {
        a: u8,
    }

    let test = <Foo as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x2a")).unwrap();
    t::assert_eq!(test.a.a, 42);
    let mut output = binrw::io::Cursor::new(t::vec![]);
    binrw::BinWrite::write(&test, &mut output).unwrap();
    t::assert_eq!(output.into_inner(), b"\x2a");
}

#[test]
fn raw_ident() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead)]
    struct Test {
        r#type: u32,
    }

    <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(t::vec![0x00, 0x00, 0x00, 0x00]))
        .unwrap();
}

#[test]
fn reader_var() {
    struct Checksum<T> {
        inner: T,
        check: ::core::num::Wrapping<u8>,
    }

    impl<T> Checksum<T> {
        fn new(inner: T) -> Self {
            Self {
                inner,
                check: ::core::num::Wrapping(0),
            }
        }

        fn check(&self) -> u8 {
            self.check.0
        }
    }

    impl<T: binrw::io::Read> binrw::io::Read for Checksum<T> {
        fn read(&mut self, buf: &mut [u8]) -> binrw::io::Result<usize> {
            let size = self.inner.read(buf)?;
            for b in &buf[0..size] {
                self.check += b;
            }
            t::Ok(size)
        }
    }

    impl<T: binrw::io::Seek> binrw::io::Seek for Checksum<T> {
        fn seek(&mut self, pos: binrw::io::SeekFrom) -> binrw::io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(little, stream = r, map_stream = Checksum::new)]
    struct Test {
        a: u16,
        b: u16,
        #[br(calc(r.check()))]
        c: u8,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\x01\x02\x03\x04")).unwrap(),
        Test {
            a: 0x201,
            b: 0x403,
            c: 10,
        }
    );
}

#[test]
fn top_level_assert_has_self() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    #[br(assert(self.verify(), "verify failed"))]
    struct Test {
        a: u8,
        b: u8,
    }

    impl Test {
        fn verify(&self) -> bool {
            self.a == self.b
        }
    }

    let mut data = binrw::io::Cursor::new(b"\x01\x01");
    <Test as binrw::BinRead>::read_le(&mut data).expect("a == b passed");
    let mut data = binrw::io::Cursor::new(b"\x01\x02");
    let err = <Test as binrw::BinRead>::read_le(&mut data).expect_err("a == b failed");
    t::assert!(t::matches!(err, binrw::Error::AssertFail {
        message,
        ..
    } if message == "verify failed"));
}

#[test]
fn top_level_assert_self_err_output_not_transformed() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    #[br(assert(self.verify()))]
    struct Test {
        a: u8,
        b: u8,
    }

    impl Test {
        fn verify(&self) -> bool {
            self.a == self.b
        }
    }

    let mut data = binrw::io::Cursor::new(b"\x01\x01");
    <Test as binrw::BinRead>::read_le(&mut data).expect("a == b passed");
    let mut data = binrw::io::Cursor::new(b"\x01\x02");
    let err = <Test as binrw::BinRead>::read_le(&mut data).expect_err("a == b failed");
    t::assert!(t::matches!(err, binrw::Error::AssertFail {
        message,
        ..
    } if message == "assertion failed: `self.verify()`"));
}

#[test]
fn top_level_assert_self_weird() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    #[br(assert(Test::verify(&self), "verify failed"))]
    struct Test {
        a: u8,
        b: u8,
    }

    impl Test {
        fn verify(&self) -> bool {
            self.a == self.b
        }
    }

    let mut data = binrw::io::Cursor::new(b"\x01\x01");
    <Test as binrw::BinRead>::read_le(&mut data).expect("a == b passed");
    let mut data = binrw::io::Cursor::new(b"\x01\x02");
    let err = <Test as binrw::BinRead>::read_le(&mut data).expect_err("a == b failed");
    t::assert!(t::matches!(err, binrw::Error::AssertFail {
        message,
        ..
    } if message == "verify failed"));
}

#[test]
fn rewind_on_assert() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    #[br(assert(b == 1))]
    struct Test {
        a: u8,
        b: u8,
    }

    let mut data = binrw::io::Cursor::new(b"\0\0\0");
    let expected = binrw::io::Seek::seek(&mut data, binrw::io::SeekFrom::Start(1)).unwrap();
    <Test as binrw::BinRead>::read_le(&mut data).expect_err("accepted bad data");
    t::assert_eq!(
        expected,
        binrw::io::Seek::stream_position(&mut data).unwrap()
    );
}

#[test]
fn rewind_on_eof() {
    #[derive(binrw::BinRead, Debug)]
    struct Test {
        _a: u8,
        // Fail on the second field to actually test that a rewind happens to
        // the beginning of the struct, not just the beginning of the field
        _b: u16,
    }

    let mut data = binrw::io::Cursor::new(b"\0\0\0");
    let expected = binrw::io::Seek::seek(&mut data, binrw::io::SeekFrom::Start(1)).unwrap();
    <Test as binrw::BinRead>::read_le(&mut data).expect_err("accepted bad data");
    t::assert_eq!(
        expected,
        binrw::io::Seek::stream_position(&mut data).unwrap()
    );
}

#[test]
fn rewind_on_field_assert() {
    #[allow(dead_code)]
    #[derive(binrw::BinRead, Debug)]
    struct Test {
        a: u8,
        // Assert on the second field to actually test that a rewind happens to
        // the beginning of the struct, not just the beginning of the field
        #[br(assert(b == 1))]
        b: u8,
    }

    let mut data = binrw::io::Cursor::new(b"\0\0\0");
    let expected = binrw::io::Seek::seek(&mut data, binrw::io::SeekFrom::Start(1)).unwrap();
    <Test as binrw::BinRead>::read_le(&mut data).expect_err("accepted bad data");
    t::assert_eq!(
        expected,
        binrw::io::Seek::stream_position(&mut data).unwrap()
    );
}

#[test]
fn try_directive() {
    #[derive(binrw::BinRead)]
    #[br(big)]
    struct Test {
        #[br(try)]
        a: t::Option<[i32; 2]>,
    }

    let result = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\0\0\0")).unwrap();
    assert!(result.a.is_none());
    let result =
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\xff\xff\xff\xff\0\0\0\0"))
            .unwrap();
    t::assert_eq!(result.a, t::Some([-1, 0]));
}

#[test]
fn try_calc() {
    #[derive(binrw::BinRead, Debug, PartialEq)]
    #[br(big, import(v: u32))]
    struct Test {
        #[br(try_calc = t::TryFrom::try_from(v))]
        a: u16,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b""), (1,)).unwrap(),
        Test { a: 1 }
    );
    <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b""), (0x1_0000,)).unwrap_err();
}

#[test]
fn tuple() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    struct Test(#[br(big)] u16, #[br(little)] u16);

    let result =
        <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new("\0\x01\x02\0")).unwrap();
    t::assert_eq!(result, Test(1, 2));
}

#[test]
fn tuple_calc_temp_field() {
    #[binrw::binread]
    #[derive(Debug, Eq, PartialEq)]
    #[br(big)]
    struct Test(#[br(temp)] u16, #[br(calc(t::Into::into(self_0 + 1)))] u32);

    let result = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x04")).unwrap();
    // This also indirectly checks that `temp` is actually working since
    // compilation would fail if it weren’t due to missing a second item
    t::assert_eq!(result, Test(5u32));
}

#[test]
fn parse_with_fn_once_closure_args() {
    #[derive(binrw::BinRead)]
    #[br(little)]
    struct Test {
        #[br(args(1), parse_with = |_, _, (a,)| t::Ok(a))]
        a: u8,
    }
    let result = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"")).unwrap();
    t::assert_eq!(result.a, 1);
}

#[test]
fn no_clone_needed_for_parse_with() {
    #[binrw::binread]
    #[derive(Debug, Eq, PartialEq)]
    struct Test {
        foo: u8,
        #[br(parse_with = files_parser, args(&mut foo))]
        files: u8,
    }

    #[binrw::parser]
    fn files_parser(foo: &mut u8) -> binrw::BinResult<u8> {
        let old_value = *foo;
        *foo = 0x0A;
        t::Ok(old_value)
    }

    let result = <Test as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x0B")).unwrap();
    t::assert_eq!(
        result,
        Test {
            foo: 0x0A,
            files: 0x0B
        }
    );
}
