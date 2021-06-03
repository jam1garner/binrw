use binrw::{BinRead, binread};

#[derive(BinRead)]
enum Foo {}

#[binread]
enum Bar {}

fn main() {}
