use binread_derive::BinRead;

#[derive(BinRead)]
#[br(big)]
struct Test {
    foo: u32
}
