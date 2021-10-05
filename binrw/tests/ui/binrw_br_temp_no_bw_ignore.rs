use binrw::binrw;

#[binrw]
struct Test {
    #[br(temp)]
    len: u32,
}

fn main() {}
