extern crate trybuild;

#[test]
fn run_all() {
    // Pass the `neon_profile` cfg flag down into trybuild's nested calls to cargo.
    std::env::set_var(
        "RUSTFLAGS",
        &format!("--cfg neon_profile={:?}", neon::meta::BUILD_PROFILE),
    );

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
