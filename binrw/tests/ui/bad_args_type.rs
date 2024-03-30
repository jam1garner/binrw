use binrw::binrw;

#[binrw]
#[br(import { _a: u8 })]
struct Item;

#[binrw]
struct Data {
    a: u8,
    b: u8,
    #[br(args(a))]
    c: Item,
}

fn main() {}
