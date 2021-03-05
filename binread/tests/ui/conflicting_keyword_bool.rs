use binread::BinRead;

#[derive(BinRead)]
struct Foo {
    #[br(restore_position, restore_position)]
    a: i32,
}

fn main() {}
