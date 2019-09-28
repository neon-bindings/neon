use std::env;

fn main() {
    if let Ok(profile) = env::var("PROFILE") {
        eprintln!("neon/build.rs: setting neon_profile={}", profile);
        println!("cargo:rustc-cfg=neon_profile={:?}", profile);
    } else {
        eprintln!("neon/build.rs: NOT setting neon_proifle");
    }
}
