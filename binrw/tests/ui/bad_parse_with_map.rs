use binrw::{helpers::until_eof, BinRead};

#[derive(BinRead)]
struct Foo {
    #[br(parse_with = until_eof::<Vec<u8>, _, _, _>, map = String::from_utf8_lossy)]
    a: String,
}

fn main() {}
