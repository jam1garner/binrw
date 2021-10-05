use binrw::BinWrite;

#[derive(BinWrite)]
struct Foo {
    #[bw(calc = 0u8)]
    a: u8,
}

fn main() {}
