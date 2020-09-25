#[cfg(feature = "legacy-runtime")]
fn main() {
    if let Ok(profile) = std::env::var("PROFILE") {
        println!("cargo:rustc-cfg=neon_profile={:?}", profile);
    }
}

#[cfg(feature = "napi-runtime")]
fn main() {}
