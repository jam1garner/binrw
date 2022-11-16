use binrw::{BinRead, BinResult};

#[binrw::parser]
fn wrong() -> BinResult<bool> {
    Ok(true)
}

#[derive(BinRead)]
struct Foo {
    #[br(parse_with = 56)]
    a: i32,
    #[br(parse_with = |_, _, _| { true })]
    b: i32,
    #[br(parse_with = |_, _, _| { Ok(true) })]
    c: i32,
    #[br(parse_with = wrong)]
    d: i32,
    #[br(parse_with = missing)]
    e: i32,
}

fn main() {}
