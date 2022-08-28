use binrw::BinrwNamedArgs;

#[derive(Eq, PartialEq, Debug)]
struct NotClone;

#[derive(BinrwNamedArgs)]
struct Test<T: Clone> {
    blah: u32,
    not_copy: String,
    not_clone: NotClone,
    #[named_args(default = 2)]
    default_val: u32,
    generic: T,
}

#[test]
fn args_macro_trailing_comma() {
    #[rustfmt::skip]
    let x: Test<String> = binrw::args! {
        blah: 3,
        not_copy: "a string here".into(),
        not_clone: NotClone,
        generic: "generic string :o".into(),
    };

    assert_eq!(x.blah, 3);
    assert_eq!(x.not_copy, "a string here");
    assert_eq!(x.not_clone, NotClone);
    assert_eq!(x.generic, "generic string :o");
    assert_eq!(x.default_val, 2);
}

#[test]
fn test() {
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
