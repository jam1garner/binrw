use binread::BinRead;

#[derive(BinRead)]
struct Struct {
    #[br(invalid_struct_field_keyword)]
    field: i32,
}

fn main() {}
