use binrw::NamedArgs;

#[derive(Eq, PartialEq, Debug)]
struct NotClone;

// Non-standard struct named Option intentionally shadows std::option::Option from the prelude
#[allow(dead_code)]
struct Option;

#[derive(NamedArgs)]
struct Test<'a, T: Clone, const N: usize> {
    blah: u32,
    not_copy: String,
    not_clone: NotClone,
    #[named_args(default = 2)]
    default_val: u32,
    generic: T,
    borrow: &'a T,
    array: [u8; N],
}

#[test]
fn args_macro_trailing_comma() {
    let s = String::new();
    let x: Test<String, 2> = binrw::args! {
        blah: 3,
        not_copy: "a string here".into(),
        not_clone: NotClone,
        generic: "generic string :o".into(),
        borrow: &s,
        array: [42; 2]
    };

    assert_eq!(x.blah, 3);
    assert_eq!(x.not_copy, "a string here");
    assert_eq!(x.not_clone, NotClone);
    assert_eq!(x.generic, "generic string :o");
    assert_eq!(x.default_val, 2);
    assert_eq!(x.borrow, &s);
    assert_eq!(x.array, [42; 2]);
}

#[test]
fn test() {
    let s = String::new();
    let x = Test::<String, 2>::builder()
        .blah(3)
        .not_copy("a string here".into())
        .not_clone(NotClone)
        .generic("generic string :o".into())
        .borrow(&s)
        .array([42; 2])
        .finalize();

    assert_eq!(x.blah, 3);
    assert_eq!(x.not_copy, "a string here");
    assert_eq!(x.not_clone, NotClone);
    assert_eq!(x.generic, "generic string :o");
    assert_eq!(x.default_val, 2);
    assert_eq!(x.borrow, &s);
    assert_eq!(x.array, [42; 2]);
}
