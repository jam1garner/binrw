use binread::{
    prelude::*,
    FilePtr,
    io::*,
    NullString
};

const TEST_CONTENTS: &[u8] = include_bytes!("./test_file2.bin");

// Effectively a regression test to make sure that immediately derefing doesn't cause
// any issues with the reader position not being restored
#[derive(BinRead, Debug)]
#[br(big, magic = b"TEST")]
struct TestFile {
    #[br(deref_now)]
    ptr: FilePtr<u32, NullString>,
    value: i32,
    #[br(calc = ptr.len())]
    ptr_len: usize,

    // try test
    #[br(try)]
    test: Option<[u64; 30]>,
}

#[test]
fn parse_test() {
    let mut reader = Cursor::new(TEST_CONTENTS);
    let test: TestFile = reader.read_be().unwrap();
    dbg!(test);
}