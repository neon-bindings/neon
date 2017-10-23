use std::env;

fn main() {
    // Capture the PROFILE environment variable to determine whether this is
    // a debug or release build. This allows us to pass this information into
    // the tests so we know how to inform compiletest_rs of the right compiler
    // flags to know which directory to find dependencies in.

    // This information is then extracted in src/lib.rs and shared as a public
    // constant string, so the integration tests can access it.
    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=neon_profile={:?}", profile);
    }
}
