extern crate cfg_if;
#[cfg(all(windows, not(feature = "neon-sys")))]
extern crate ureq;

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
    } else if #[cfg(windows)] {
        // ^ automatically not neon-sys
        use std::fs::File;
        use std::io::{self, Read, ErrorKind};
        use std::process::Command;

        fn node_version() -> io::Result<String> {
            let output = Command::new("node").arg("-v").output()?;
            let stdout = String::from_utf8(output.stdout).map_err(|error| {
                io::Error::new(ErrorKind::InvalidData, error)
            })?;
            Ok(stdout.trim().to_string())
        }

        fn download_node_lib(version: &str) -> io::Result<impl Read> {
            let mut request = ureq::get(&format!(
                "https://nodejs.org/dist/{version}/win-{arch}/node.lib",
                version = version,
                arch = "x64",
            ));
            let response = request.call();
            Ok(response.into_reader())
        }

        /// Set up the build environment by setting Cargo configuration variables.
        pub fn setup() {
            let version = node_version().expect("Could not determine Node.js version");
            let mut node_lib = download_node_lib(&version).expect("Could not connect to nodejs.org");
            let mut output = File::create(concat!(env!("OUT_DIR"), r"\node.lib")).unwrap();
            std::io::copy(&mut node_lib, &mut output);

            println!("cargo:rustc-link-search=native={}", env!("OUT_DIR"));
            // println!("cargo:rustc-link-search=native={}", &node_lib_path.display());
            println!("cargo:rustc-link-lib=node");
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
