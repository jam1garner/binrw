use binread::BinRead;

#[derive(BinRead)]
enum Enum {
    #[br(invalid_enum_variant_keyword)]
    A(i32),
}

fn main() {}
