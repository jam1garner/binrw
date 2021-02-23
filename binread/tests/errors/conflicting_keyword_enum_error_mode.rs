use binread::BinRead;

#[derive(BinRead)]
#[br(return_all_errors, return_unexpected_error)]
enum Foo {
    A(i32),
}

fn main() {}
