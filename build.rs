use std::env;

fn main() {
    if cfg!(windows) {
        println!("cargo:node_root_dir={}", env::var("DEP_NEON_SYS_NODE_ROOT_DIR").unwrap());
        println!("cargo:node_arch={}", env::var("DEP_NEON_SYS_NODE_ARCH").unwrap());
        println!("cargo:node_lib_file={}", env::var("DEP_NEON_SYS_NODE_LIB_FILE").unwrap());
    }

    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=neon_profile={:?}", profile);
    }
}
