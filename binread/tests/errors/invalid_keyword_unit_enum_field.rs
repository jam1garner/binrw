use binread::BinRead;

#[derive(BinRead)]
#[br(repr = u8)]
enum UnitEnum {
    #[br(invalid_unit_enum_field_keyword)]
    A,
}

fn main() {}
