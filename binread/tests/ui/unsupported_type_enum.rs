use binread::{BinRead, derive_binread};

#[derive(BinRead)]
enum Foo {}

#[derive_binread]
enum Bar {}

fn main() {}
