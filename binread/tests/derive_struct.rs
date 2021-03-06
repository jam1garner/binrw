use binread::{BinRead, BinResult, derive_binread, io::{Cursor, Read, Seek, SeekFrom}, FilePtr, NullString, ReadOptions};

#[test]
fn all_the_things() {
    #[derive(Debug)]
    struct PlainObject;

    #[derive(BinRead, Debug)]
    #[br(big, magic = b"TEST")]
    #[br(assert(entries.len() as u32 == extra_entry_count + 1))]
    struct Test {
        extra_entry_count: u32,

        #[br(count = extra_entry_count + 1, args(0x69))]
        entries: Vec<FilePtr<u32, TestEntry>>,

        #[br(default)]
        start_as_none: Option<PlainObject>,

        #[br(calc = 1 + 2)]
        calc_test: u32
    }

    fn read_offsets<R: Read + Seek>(reader: &mut R, ro: &ReadOptions, _: ())
        -> BinResult<(u16, u16)>
    {
        Ok((
            u16::read_options(reader, ro, ())?,
            u16::read_options(reader, ro, ())?
        ))
    }

    #[derive(BinRead, Debug)]
    #[br(little, magic = b"TST2")]
    #[br(import(extra_val: u8))]
    struct TestEntry {
        #[br(map = |val: u32| val.to_string())]
        entry_num: String,

        #[br(assert(offsets.1 - offsets.0 == 0x10))]
        #[br(seek_before(SeekFrom::Current(4)))]
        #[br(parse_with = read_offsets)]
        #[br(is_big = entry_num == "1")]
        offsets: (u16, u16),

        #[br(if(offsets.0 == 0x20))]
        name: Option<FilePtr<u32, NullString>>,

        #[br(calc(extra_val))]
        extra_val: u8,
    }

    Test::read(&mut Cursor::new(include_bytes!("data/test_file.bin"))).unwrap();
}

#[test]
fn assert() {
    #[derive(BinRead, Debug)]
    struct Test {
        #[br(assert(a == 1))]
        a: u8,
    }

    Test::read(&mut Cursor::new("\x01")).unwrap();
    let error = Test::read(&mut Cursor::new("\0")).expect_err("accepted bad data");
    match error {
        binread::Error::AssertFail { pos, message } => {
            assert_eq!(pos, 0);
            assert_eq!(message, "a == 1");
        },
        _ => panic!("bad error type"),
    }
}

#[test]
fn assert_custom_err() {
    #[derive(Debug)]
    struct Oops(u8);

    #[derive(BinRead, Debug)]
    struct Test {
        #[br(assert(a == 1, Oops(a)))]
        a: u8,
    }

    Test::read(&mut Cursor::new("\x01")).unwrap();
    let error = Test::read(&mut Cursor::new("\x02")).expect_err("accepted bad data");
    let error = error.custom_err::<Oops>().expect("bad error type");
    assert_eq!(error.0, 2);
}

#[test]
fn assert_formatted() {
    #[derive(BinRead, Debug)]
    struct Test {
        #[br(assert(a == 1, "a was {}", a))]
        a: u8,
    }

    Test::read(&mut Cursor::new("\x01")).unwrap();
    let error = Test::read(&mut Cursor::new("\0")).expect_err("accepted bad data");
    match error {
        binread::Error::AssertFail { pos, message } => {
            assert_eq!(pos, 0);
            assert_eq!(message, "a was 0");
        },
        _ => panic!("bad error type"),
    }
}

#[test]
fn calc_temp_field() {
    #[derive_binread]
    #[derive(Debug, PartialEq)]
    #[br(big)]
    struct Test {
        #[br(temp)]
        len: u32,

        #[br(count = len)]
        vec: Vec<u8>,
    }

    let result = Test::read(&mut Cursor::new(b"\0\0\0\x05ABCDE")).unwrap();
    // This also indirectly checks that `temp` is actually working since
    // compilation would fail if it weren’t due to missing the `len` field
    assert_eq!(result, Test { vec: Vec::from(&b"ABCDE"[..]) });
}

#[test]
fn calc_try() {
    #[derive(BinRead, Debug, PartialEq)]
    #[br(big)]
    struct Test {
        #[br(calc(1), try)]
        a: Option<i32>,
    }

    let result = Test::read(&mut Cursor::new(b"")).unwrap();
    assert_eq!(result, Test { a: Some(1) });
}

#[test]
fn deref_now() {
    #[derive(BinRead, Debug, PartialEq)]
    #[br(big, magic = b"TEST")]
    struct Test {
        // deref_now on the first field tests that the reader position is correctly
        // restored before reading the second field
        #[br(deref_now)]
        a: FilePtr<u32, NullString>,
        b: i32,
    }

    let result = Test::read(&mut Cursor::new(include_bytes!("data/deref_now.bin"))).unwrap();
    assert_eq!(result, Test {
        a: FilePtr { ptr: 0x10, value: Some(NullString(b"Test string".to_vec())) },
        b: -1,
    });
}

#[test]
fn empty_imports() {
    #[derive(BinRead, Debug, PartialEq)]
    #[br(import())]
    struct Test {
        a: u8,
    }

    let result = Test::read(&mut Cursor::new(b"\x01")).unwrap();
    assert_eq!(result, Test { a: 1 });
}

#[test]
fn ignore_and_default() {
    #[derive(Debug, Eq, PartialEq)]
    struct One(u8);
    impl Default for One {
        fn default() -> Self {
            Self(1)
        }
    }

    #[derive(BinRead, Debug, PartialEq)]
    #[br(big)]
    struct Test {
        a: u8,
        #[br(default)]
        b: One,
        #[br(ignore)]
        c: One,
    }

    let result = Test::read(&mut Cursor::new(b"\x02")).unwrap();
    assert_eq!(result, Test { a: 2, b: <_>::default(), c: <_>::default() });
}

#[test]
fn magic_byte() {
    #[derive(BinRead, Debug)]
    #[br(magic = b'a')]
    struct Test;

    Test::read(&mut Cursor::new(b"a")).unwrap();
    Test::read(&mut Cursor::new(b"")).expect_err("accepted bad data");
    Test::read(&mut Cursor::new(b"x")).expect_err("accepted bad data");
}

#[test]
fn magic_char() {
    #[derive(BinRead, Debug)]
    #[br(magic = 'a')]
    struct Test;

    Test::read(&mut Cursor::new(b"a")).unwrap();
    Test::read(&mut Cursor::new(b"")).expect_err("accepted bad data");
    Test::read(&mut Cursor::new(b"x")).expect_err("accepted bad data");
}

#[test]
fn magic_field() {
    #[derive(BinRead, Debug, PartialEq)]
    #[br(magic(b"A"))]
    struct Test {
        b: u8,
        #[br(magic(b"C"))]
        d: u8,
    }

    Test::read(&mut Cursor::new(b"ABBB")).expect_err("accepted bad data");
    let result = Test::read(&mut Cursor::new(b"ABCD")).unwrap();
    assert_eq!(result, Test { b: b'B', d: b'D' });
}

#[test]
fn pad_after_before() {
    #[derive(BinRead, Debug, PartialEq)]
    struct Test {
        #[br(pad_after = 1, pad_before = 1)]
        a: u8,
        b: u8,
    }

    let result = Test::read(&mut Cursor::new(b"\0\x01\0\x02")).unwrap();
    assert_eq!(result, Test { a: 1, b: 2 });
}

#[test]
fn import_tuple() {
    #[derive(BinRead, Debug)]
    struct Test {
        #[br(args_tuple = (1, 2))]
        a: Child,
    }

    #[derive(BinRead, Debug)]
    #[br(import_tuple(args: (u8, u8)))]
    struct Child {
        #[br(calc(args.0 + args.1))]
        a: u8,
    }

    let result = Test::read(&mut Cursor::new(b"")).unwrap();
    assert_eq!(result.a.a, 3);
}

#[test]
fn offset_after() {
    #[derive(BinRead, Debug)]
    struct Test {
        #[br(offset_after = b.into())]
        a: FilePtr<u8, u8>,
        b: u8,
    }

    let result = Test::read(&mut Cursor::new(b"\x01\x03\xff\xff\x04")).unwrap();
    assert_eq!(*result.a, 4);
}

#[test]
fn rewind_on_assert() {
    #[derive(BinRead, Debug)]
    #[br(assert(b == 1))]
    struct Test {
        a: u8,
        b: u8,
    }

    let mut data = Cursor::new(b"\0\0\0");
    let expected = data.seek(SeekFrom::Start(1)).unwrap();
    Test::read(&mut data).expect_err("accepted bad data");
    assert_eq!(expected, data.seek(SeekFrom::Current(0)).unwrap());
}

#[test]
fn rewind_on_eof() {
    #[derive(BinRead, Debug)]
    struct Test {
        a: u8,
        // Fail on the second field to actually test that a rewind happens to
        // the beginning of the struct, not just the beginning of the field
        b: u16,
    }

    let mut data = Cursor::new(b"\0\0\0");
    let expected = data.seek(SeekFrom::Start(1)).unwrap();
    Test::read(&mut data).expect_err("accepted bad data");
    assert_eq!(expected, data.seek(SeekFrom::Current(0)).unwrap());
}

#[test]
fn rewind_on_field_assert() {
    #[derive(BinRead, Debug)]
    struct Test {
        a: u8,
        // Assert on the second field to actually test that a rewind happens to
        // the beginning of the struct, not just the beginning of the field
        #[br(assert(b == 1))]
        b: u8,
    }

    let mut data = Cursor::new(b"\0\0\0");
    let expected = data.seek(SeekFrom::Start(1)).unwrap();
    Test::read(&mut data).expect_err("accepted bad data");
    assert_eq!(expected, data.seek(SeekFrom::Current(0)).unwrap());
}

#[test]
fn try_directive() {
    #[derive(BinRead)]
    #[br(big)]
    struct Test {
        #[br(try)]
        a: Option<[ i32; 2 ]>,
    }

    let result = Test::read(&mut Cursor::new(b"\0\0\0\0")).unwrap();
    assert!(result.a.is_none());
    let result = Test::read(&mut Cursor::new(b"\xff\xff\xff\xff\0\0\0\0")).unwrap();
    assert_eq!(result.a, Some([ -1, 0 ]));
}

#[test]
fn tuple() {
    #[derive(BinRead, Debug, Eq, PartialEq)]
    struct Test(#[br(big)] u16, #[br(little)] u16);

    let result = Test::read(&mut Cursor::new("\0\x01\x02\0")).unwrap();
    assert_eq!(result, Test(1, 2));
}

#[test]
fn tuple_calc_temp_field() {
    #[derive_binread]
    #[derive(Debug, Eq, PartialEq)]
    #[br(big)]
    struct Test(#[br(temp)] u16, #[br(calc((self_0 + 1).into()))] u32);

    let result = Test::read(&mut Cursor::new(b"\0\x04")).unwrap();
    // This also indirectly checks that `temp` is actually working since
    // compilation would fail if it weren’t due to missing a second item
    assert_eq!(result, Test(5u32));
}
