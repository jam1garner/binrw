#[test]
fn errors() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/errors/*.rs");
}
