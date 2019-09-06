use std::env;
use std::path::Path;

/// Set up the build environment by setting Cargo configuration variables.
pub fn setup() {
    if cfg!(windows) {
        let debug = env::var("DEBUG").ok().map_or(false, |s| s == "true");
        let configuration = if debug { "Debug" } else { "Release" };
        let node_root_dir = env::var("DEP_NEON_RUNTIME_NODE_ROOT_DIR").unwrap();
        let node_lib_file = env::var("DEP_NEON_RUNTIME_NODE_LIB_FILE").unwrap();
        let node_arch = env::var("DEP_NEON_RUNTIME_NODE_ARCH").unwrap();
        let node_lib_file_path = Path::new(&node_lib_file);
        let mut node_lib_path = Path::new(&node_root_dir).to_path_buf();
        node_lib_path.push(&node_arch);
        println!("cargo:rustc-link-search={}\\{}", node_root_dir, configuration);
        println!("cargo:rustc-link-search=native={}", &node_lib_path.display());
        println!("cargo:rustc-link-lib={}", &node_lib_file_path.file_stem().unwrap().to_str().unwrap());

        // Link `win_delay_load_hook.obj` for windows electron
        let node_runtime_env = "npm_config_runtime";
        println!("cargo:rerun-if-env-changed={}", node_runtime_env);
        if env::var(node_runtime_env).map(|s| s == "electron") == Ok(true) {
            println!("cargo:rustc-cdylib-link-arg=win_delay_load_hook.obj");
            println!("cargo:rustc-cdylib-link-arg=delayimp.lib");
            println!("cargo:rustc-cdylib-link-arg=/DELAYLOAD:node.exe");
        }
    }
}
