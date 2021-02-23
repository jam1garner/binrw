use binread::BinRead;

#[derive(BinRead)]
union Bar {
    a: i32,
}

fn main() {}
