use binread::{BinRead, derive_binread};

#[derive(BinRead)]
union Foo {
    a: i32,
}

#[derive_binread]
union Bar {
    a: i32,
}

fn main() {}
