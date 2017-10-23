// Since Rust doesn't currently let you write build scripts for tests,
// we'll capture the PROFILE environment variable from build.rs here
// and make it available in a public constant that the tests can access.

#[cfg(neon_profile = "release")]
pub const PROFILE: &'static str = "release";

#[cfg(not(neon_profile = "release"))]
pub const PROFILE: &'static str = "debug";
