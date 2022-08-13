use binrw::BinrwNamedArgs;

#[test]
fn test() {
    #[derive(Eq, PartialEq, Debug)]
    pub struct NotClone;

    #[derive(BinrwNamedArgs)]
    pub struct Test<T: Clone> {
        blah: u32,
        not_copy: String,
        not_clone: NotClone,
        #[named_args(default = 2)]
        default_val: u32,
        generic: T,
    }

    let x = Test::<String>::builder()
        .blah(3)
        .not_copy("a string here".into())
        .not_clone(NotClone)
        .generic("generic string :o".into())
        .finalize();

    assert_eq!(x.blah, 3);
    assert_eq!(x.not_copy, "a string here");
    assert_eq!(x.not_clone, NotClone);
    assert_eq!(x.generic, "generic string :o");
    assert_eq!(x.default_val, 2);
}
