extern crate gcc;

use std::process::{Command, Stdio};
use std::env;

fn main() {
    // 1. Build the object file from source using node-gyp.
    build_object_file();

    // 2. Link the library from the object file using gcc.
    link_library();
}

fn build_object_file() {
    let npm_command = if cfg!(unix) { "npm" } else { "npm.cmd" };
    let node_gyp_command = if cfg!(unix) { "node-gyp" } else { "node-gyp.cmd" };

    if cfg!(windows) {
        // Downcase all the npm environment variables to ensure they are read by node-gyp.
        for (key, value) in env::vars() {
            if key.starts_with("NPM_CONFIG") {
                env::remove_var(&key);
                env::set_var(key.to_lowercase(), value);
            }
        }
    }

    // Ensure that all package.json dependencies and dev dependencies are installed.
    Command::new(npm_command).args(&["install", "--silent"]).status().ok().expect("Failed to run \"npm install\" for neon-sys!");

    // Run `node-gyp configure` in verbose mode to read node_root_dir on Windows.
    let mut configure_args = vec!["configure", "--verbose"];
    if debug() {
        configure_args.push("--debug");
    }

    let output = Command::new(node_gyp_command)
        .args(&configure_args)
        .output()
        .expect("Failed to run \"node-gyp configure\" for neon-sys!");

    if cfg!(windows) {
        let node_gyp_output = String::from_utf8_lossy(&output.stderr);
        let node_root_dir_flag_pattern = "'-Dnode_root_dir=";
        let node_root_dir_start_index = node_gyp_output
            .find(node_root_dir_flag_pattern)
            .map(|i| i + node_root_dir_flag_pattern.len())
            .expect("Couldn't find node_root_dir in node-gyp output.");
        let node_root_dir_end_index = node_gyp_output[node_root_dir_start_index..].find("'").unwrap() + node_root_dir_start_index;
        println!("cargo:node_root_dir={}", &node_gyp_output[node_root_dir_start_index..node_root_dir_end_index]);
        let node_lib_file_flag_pattern = "'-Dnode_lib_file=";
        let node_lib_file_start_index = node_gyp_output
            .find(node_lib_file_flag_pattern)
            .map(|i| i + node_lib_file_flag_pattern.len())
            .expect("Couldn't find node_lib_file in node-gyp output.");
        let node_lib_file_end_index = node_gyp_output[node_lib_file_start_index..].find(".lib").unwrap() + node_lib_file_start_index;
        println!("cargo:node_lib_file={}", &node_gyp_output[node_lib_file_start_index..node_lib_file_end_index]);
    }

    // Run `node-gyp build` (appending -d in debug mode).
    let mut build_args = vec!["build"];
    if debug() {
        build_args.push("--debug");
    }

    Command::new(node_gyp_command)
        .stderr(Stdio::null()) // Prevent cargo build from hanging on Windows.
        .args(&build_args)
        .status()
        .ok()
        .expect("Failed to run \"node-gyp build\" for neon-sys!");
}

// Link the built object file into a static library.
fn link_library() {
    let configuration = if debug() { "Debug" } else { "Release" };
    let object_path = if cfg!(unix) {
        format!("build/{}/obj.target/neon/src/neon.o", configuration)
    } else {
        format!("build\\{}\\obj\\neon\\neon.obj", configuration)
    };

    gcc::Config::new().object(object_path).compile("libneon.a");
}

fn debug() -> bool {
    match env::var("DEBUG") {
        Ok(s) => s == "true",
        Err(_) => false
    }
}
