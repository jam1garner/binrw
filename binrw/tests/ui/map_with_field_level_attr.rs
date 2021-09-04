use binrw::binread;

#[binread]
#[br(map = |_: u8| Test { y: 3 })]
struct Test {
    #[br(map(|x| x + 1))]
    y: u64,
}

fn main() {

}
