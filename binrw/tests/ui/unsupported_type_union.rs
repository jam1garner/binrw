use binrw::{BinRead, binread};

#[derive(BinRead)]
union Foo {
    a: i32,
}

#[binread]
union Bar {
    a: i32,
}

fn main() {}
