use binrw::{BinRead, BinReaderExt};

#[derive(Clone)]
struct OnlyCloneable;

#[derive(BinRead)]
#[br(import(needs_clone: OnlyCloneable))]
struct ArgsNeedClone {}

#[derive(BinRead)]
struct TestCloneArray {
    #[br(args(needs_clone: OnlyCloneable))]
    array: [ArgsNeedClone; 35],

    #[br(args(needs_clone: OnlyCloneable))]
    #[br(count = 4)]
    vec: Vec<ArgsNeedClone>,
}

#[test]
fn clone_args() {
    let mut x = binrw::io::Cursor::new(&[]);

    let y: TestCloneArray = x.read_be().unwrap();
}
