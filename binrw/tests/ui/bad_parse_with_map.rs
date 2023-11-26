use binrw::{helpers::until_eof, BinRead};

#[derive(BinRead)]
struct Foo {
    #[br(parse_with = until_eof::<_, _, _, Vec<u8>>, map = String::from_utf8_lossy)]
    a: String,
}

fn main() {}
