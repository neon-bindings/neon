use semver::Version;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub const MAJOR: &'static str = env!("CARGO_PKG_VERSION_MAJOR");
pub const MINOR: &'static str = env!("CARGO_PKG_VERSION_MINOR");
pub const PATCH: &'static str = env!("CARGO_PKG_VERSION_PATCH");

pub fn version() -> Version {
    Version {
        major: MAJOR.parse().unwrap(),
        minor: MINOR.parse().unwrap(),
        patch: PATCH.parse().unwrap(),
        pre: vec![],
        build: vec![]
    }
}

// We captured the build profile from build.rs and saved it in the cfg variable `neon_profile`.

#[cfg(neon_profile = "release")]
pub const BUILD_PROFILE: &'static str = "release";

#[cfg(not(neon_profile = "release"))]
pub const BUILD_PROFILE: &'static str = "debug";
