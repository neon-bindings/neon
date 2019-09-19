extern crate trybuild;

#[test]
fn run_all() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
