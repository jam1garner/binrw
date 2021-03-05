use binread::BinRead;

#[derive(BinRead)]
#[br(import(in_var: i16))]
#[br(invalid_struct_keyword)]
struct Test {
    a: i16,
}

fn main() {
    // There should be no error from the compiler that the passed argument type
    // is wrong, since parsing of the struct was successful enough that the type
    // information could be resolved
    Test::read_args(&mut binread::io::Cursor::new(b"\0\0"), (1, )).unwrap();
}
