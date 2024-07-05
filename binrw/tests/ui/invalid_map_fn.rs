#![allow(dependency_on_unit_never_type_fallback)]
use binrw::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(map = does_not_exist)]
    a: i32,
}

fn main() {}
