extern crate trybuild;
extern crate rustversion;

// The compiler errors are sensitive to the Rust version, so we'll keep
// them updated for recent nightlies. This also allows us to catch any
// usability regressions.
#[rustversion::nightly]
#[test]
fn run_all() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
