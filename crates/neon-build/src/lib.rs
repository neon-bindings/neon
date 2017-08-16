use std::env;
use std::path::Path;

/// Set up the build environment by setting Cargo configuration variables.
pub fn setup() {
    if cfg!(windows) {
        let node_root_dir = env::var("DEP_NEON_RUNTIME_NODE_ROOT_DIR").unwrap();
        println!("cargo:rustc-link-search=native={}", node_root_dir);
        println!("cargo:rustc-link-lib=node");
    }
}
