use std::env;

fn main() {
    if cfg!(windows) {
        let debug = env::var("DEBUG").ok().map_or(false, |s| s == "true");
        let configuration = if debug { "Debug" } else { "Release" };
        let node_root_dir = env::var("DEP_NEON_SYS_NODE_ROOT_DIR").unwrap();
        println!("cargo:rustc-link-lib=node");
        println!("cargo:rustc-link-search={}\\{}", node_root_dir, configuration);
    }
}
