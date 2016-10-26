use std::env;

fn main() {
    if cfg!(windows) {
        println!("cargo:node_root_dir={}", env::var("DEP_NEON_NODE_ROOT_DIR").unwrap());
    }
}
