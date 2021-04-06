use binrw::BinrwNamedArgs;

#[test]
fn test() {
    #[derive(PartialEq, Debug)]
    struct NotClone;

    #[derive(BinrwNamedArgs)]
    struct Test {
        blah: u32,
        not_copy: String,
        not_clone: NotClone,
    }

    let x = Test::builder()
        .blah(3)
        .not_copy("a string here".into())
        .not_clone(NotClone)
        .finalize();

    assert_eq!(x.blah, 3);
    assert_eq!(x.not_copy, "a string here");
    assert_eq!(x.not_clone, NotClone);
}
