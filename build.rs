use std::env;

fn main() {
    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=neon_profile={:?}", profile);
    }
}
