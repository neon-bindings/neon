extern crate cfg_if;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(windows, feature = "neon-sys"))] {
        use std::env::var;
        use std::path::Path;

        const NODE_ROOT_DIR: &'static str = include_str!(concat!(env!("OUT_DIR"), "\\node_root_dir"));
        const NODE_ARCH:     &'static str = include_str!(concat!(env!("OUT_DIR"), "\\node_arch"));
        const NODE_LIB_FILE: &'static str = include_str!(concat!(env!("OUT_DIR"), "\\node_lib_file"));

        /// Set up the build environment by setting Cargo configuration variables.
        pub fn setup() {
            let debug = var("DEBUG").ok().map_or(false, |s| s == "true");
            let configuration = if debug { "Debug" } else { "Release" };
            let node_lib_file_path = Path::new(NODE_LIB_FILE);
            let mut node_lib_path = Path::new(NODE_ROOT_DIR).to_path_buf();
            node_lib_path.push(NODE_ARCH);
            println!("cargo:rustc-link-search={}\\{}", NODE_ROOT_DIR, configuration);
            println!("cargo:rustc-link-search=native={}", &node_lib_path.display());
            println!("cargo:rustc-link-lib={}", &node_lib_file_path.file_stem().unwrap().to_str().unwrap());

            // Link `win_delay_load_hook.obj` for windows electron
            let node_runtime_env = "npm_config_runtime";
            println!("cargo:rerun-if-env-changed={}", node_runtime_env);
            if var(node_runtime_env).map(|s| s == "electron") == Ok(true) {
                println!("cargo:rustc-cdylib-link-arg=win_delay_load_hook.obj");
                println!("cargo:rustc-cdylib-link-arg=delayimp.lib");
                println!("cargo:rustc-cdylib-link-arg=/DELAYLOAD:node.exe");
            }
        }
    } else if #[cfg(target_os = "macos")] {
        /// Set up the build environment by setting Cargo configuration variables.
        pub fn setup() {
            println!("cargo:rustc-cdylib-link-arg=-undefined");
            println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
        }
    } else {
        pub fn setup() { }
    }
}
