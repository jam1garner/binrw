use binread::BinRead;

#[derive(BinRead)]
#[br(assert(does_not_exist == 0))]
struct Foo {
    a: i32,
}

// This tests that fields from one variant do not leak to all variants
#[derive(BinRead)]
#[br(assert(a == 0))]
enum Bar {
    A { a: i32 },
    B { b: i32 },
}

// This tests that fields from one variant do not leak to subsequent variants
#[derive(BinRead)]
enum Baz {
    A { a: i32 },
    #[br(assert(a == 0))]
    B { b: i32 },
}

fn main() {}
