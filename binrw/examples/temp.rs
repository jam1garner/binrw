use binrw::{
    args, binread,
    io::{Cursor, Read, Seek, SeekFrom},
    BinRead, BinResult, FilePtr, NullString, ReadOptions,
};

fn main() {
    #[derive(Debug)]
    struct PlainObject;

    #[derive(BinRead, Debug)]
    #[br(is_big = true, magic = b"TEST")]
    //#[br(assert(entries.len() as u32 == extra_entry_count + 1))]
    struct Test {
        extra_entry_count: u32,

        //#[br(count = extra_entry_count + 1, args { inner: args! { extra_val: 0x69 } })]
        //entries: Vec<FilePtr<u32, TestEntry>>,
        #[br(default)]
        start_as_none: Option<PlainObject>,

        #[br(calc = 1 + 2)]
        calc_test: u32,
    }

    fn read_offsets<R: Read + Seek>(
        reader: &mut R,
        ro: &ReadOptions,
        _: (),
    ) -> BinResult<(u16, u16)> {
        Ok((
            u16::read_options(reader, ro, ())?,
            u16::read_options(reader, ro, ())?,
        ))
    }

    #[derive(BinRead, Debug)]
    #[br(little, magic = b"TST2")]
    #[br(import { extra_val: u8 })]
    struct TestEntry {
        #[br(map = |val: u32| val.to_string())]
        entry_num: String,

        #[br(assert(offsets.1 - offsets.0 == 0x10))]
        #[br(seek_before(SeekFrom::Current(4)))]
        #[br(parse_with = read_offsets)]
        #[br(is_big = entry_num == "1")]
        offsets: (u16, u16),

        //#[br(if(offsets.0 == 0x20))]
        //name: Option<FilePtr<u32, NullString>>,
        #[br(calc(extra_val))]
        extra_val: u8,
    }

    //Test::read(&mut Cursor::new(include_bytes!("data/test_file.bin"))).unwrap();
}
