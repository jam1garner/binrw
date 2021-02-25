use binread::{BinRead, BinResult, io::{Cursor, Read, Seek, SeekFrom}, FilePtr, NullString, ReadOptions};

use binread::BinReaderExt;

#[derive(Debug)]
struct BadDifferenceError(u16);

#[derive(BinRead, Debug)]
#[br(big, magic = b"TEST")]
#[br(assert(entries.len() as u32 == extra_entry_count + 1))]
struct TestFile {
    extra_entry_count: u32,

    #[br(count = extra_entry_count + 1, args(0x69))]
    entries: Vec<FilePtr<u32, TestEntry>>,

    #[br(default)]
    start_as_none: Option<NotBinWrite>,

    #[br(calc = 1 + 2)]
    calc_test: u32
}

#[derive(Debug)]
struct NotBinWrite {}

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

    #[br(seek_before(SeekFrom::Current(4)))]
    #[br(parse_with = read_offsets)]
    //#[br(is_big = "entry_num == \"1\"")]
    offsets: (u16, u16),

    #[br(assert(
          /*assertion = */ offsets.1 - offsets.0 == 0x10,
        /*raise error = */ BadDifferenceError(offsets.1 - offsets.0)
    ))]
    #[br(if(offsets.0 == 0x20))]
    name: Option<FilePtr<u32, NullString>>,

    #[br(ignore)]
    extra_val: u8,
}

const TEST_CONTENTS: &[u8] = include_bytes!("./test_file.bin");

#[test]
fn test_read() {
    let mut test = Cursor::new(TEST_CONTENTS);
    let test_file: TestFile = test.read_le().unwrap();
    dbg!(test_file);
}

const BAD_TEST_CONTENTS: &[u8] = include_bytes!("./test_file_bad.bin");

#[test]
fn test_assert_fail() {
    let mut test = Cursor::new(BAD_TEST_CONTENTS);
    let err = test.read_le::<TestFile>()
        .expect_err("Offset assertion should have failed");
    let custom_err = err.custom_err::<BadDifferenceError>().expect("Error type was lost");
    assert_eq!(custom_err.0, 0xBAAD - 0x20, "Unexpected failure value");
}

#[derive(BinRead, Debug)]
#[br(big, magic = b"TEST")]
struct TestTupleStruct (
    u32,

    #[br(count = self_0 + 1, args(0x69))]
    Vec<FilePtr<u32, TestEntry>>,

    #[br(default)]
    Option<NotBinWrite>,

    #[br(calc = 1 + 2)]
    u32
);

#[test]
fn test_tuple() {
    let mut test = Cursor::new(TEST_CONTENTS);
    dbg!(TestTupleStruct::read(&mut test).unwrap());
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

    let mut reader = Cursor::new(include_bytes!("deref_now.bin"));
    let result = Test::read(&mut reader).unwrap();
    assert_eq!(result, Test {
        a: FilePtr { ptr: 0x10, value: Some(NullString(b"Test string".to_vec())) },
        b: -1,
    });
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
