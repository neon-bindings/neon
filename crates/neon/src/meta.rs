//! Metadata about the Neon version and build.

use semver::Version;

/// The Neon version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The Neon major version.
pub const MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");

/// The Neon minor version.
pub const MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");

/// The Neon patch version.
pub const PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");

/// Produces a `semver::Version` data structure representing the Neon version.
pub fn version() -> Version {
    Version {
        major: MAJOR.parse().unwrap(),
        minor: MINOR.parse().unwrap(),
        patch: PATCH.parse().unwrap(),
        pre: vec![],
        build: vec![],
    }
}
