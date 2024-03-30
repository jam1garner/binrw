use binrw::binrw;

#[binrw]
struct Test {
    #[br(temp)]
    _len: u32,
}

fn main() {}
