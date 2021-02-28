use binread::BinRead;

// Errors on one field should not prevent the parser from surfacing errors on
// other fields
#[derive(BinRead)]
#[br(invalid_keyword_struct)]
struct Foo {
    #[br(invalid_keyword_struct_field_a)]
    a: i32,
    #[br(invalid_keyword_struct_field_b)]
    b: i32,
}

fn main() {}
