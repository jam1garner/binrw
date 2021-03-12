// Currently running these tests on nightly compiler only because:
// 1. They are very slow;
// 2. Output varies according to compiler version, and is less good on stable
//    (~1.50) compilers due to missing support for `proc_macro_span`.
// In the future, possibly when proc macro diagnostic enhancements are
// stabilised and https://github.com/dtolnay/trybuild/issues/6 is fixed, running
// these tests all the time makes sense.
#[rustversion::nightly]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
