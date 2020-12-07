#[cfg(feature = "neon-sys")]
mod legacy;
#[cfg(not(feature = "neon-sys"))]
mod napi;

#[cfg(not(feature = "neon-sys"))]
pub use napi::Builder;

/// Custom build scripts for [Neon][neon] modules.
/// Must be called from `main.rs` in a Cargo [build script][build-script].
///
/// ```toml
/// [package]
/// build = "build.rs"
/// ```
///
/// ```rust
/// // build.rs
/// # #[allow(clippy::needless_doctest_main)]
/// fn main() {
///     neon_build::setup();
/// }
/// ```
///
/// [neon]: https://docs.rs/neon
/// [build-script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html
pub fn setup() {
    #[cfg(feature = "neon-sys")]
    legacy::setup();
    #[cfg(not(feature = "neon-sys"))]
    napi::setup();
}
