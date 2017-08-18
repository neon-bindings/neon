extern crate gcc;
extern crate regex;

use std::process::Command;
use std::env;
use regex::Regex;

fn main() {
    // 1. Build the object file from source using node-gyp.
    build_object_file();

    // 2. Link the library from the object file using gcc.
    link_library();
}

#[cfg(unix)]
fn npm() -> Command {
    Command::new("npm")
}

#[cfg(windows)]
fn npm() -> Command {
    let mut cmd = Command::new("cmd.exe");
    cmd.args(&["/C", "npm"]);
    cmd
}

fn build_object_file() {
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
    npm().args(&["install", "--silent"]).status().ok().expect("Failed to run \"npm install\" for neon-runtime!");

    // Run `node-gyp configure` in verbose mode to read node_root_dir on Windows.
    let output = npm()
        .args(&["run", if debug() { "configure-debug" } else { "configure-release" }])
        .output()
        .expect("Failed to run \"node-gyp configure\" for neon-runtime!");

    if cfg!(windows) {
        let node_gyp_output = String::from_utf8_lossy(&output.stderr);
        let version_regex = Regex::new(r"node@(?P<version>\d+\.\d+\.\d+)\s+\|\s+(?P<platform>\w+)\s+\|\s(?P<arch>ia32|x64)").unwrap();
        let captures = version_regex.captures(&node_gyp_output).unwrap();
        println!("cargo:node_arch={}", &captures["arch"]);
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
        let node_lib_file_end_index = node_gyp_output[node_lib_file_start_index..].find("'").unwrap() + node_lib_file_start_index;
        let node_lib_file = &node_gyp_output[node_lib_file_start_index..node_lib_file_end_index];
        println!("cargo:node_lib_file={}", node_lib_file);
    }

    // Run `node-gyp build`.
    npm()
        .args(&["run", if debug() { "build-debug" } else { "build-release" }])
        .status()
        .ok()
        .expect("Failed to run \"node-gyp build\" for neon-runtime!");
}

// Link the built object file into a static library.
fn link_library() {
    let configuration = if debug() { "Debug" } else { "Release" };
    let object_path = if cfg!(unix) {
        format!("build/{}/obj.target/neon/src/neon.o", configuration)
    } else {
        format!("build\\{}\\obj\\neon\\neon.obj", configuration)
    };

    gcc::Build::new().object(object_path).compile("libneon.a");
}

fn debug() -> bool {
    match env::var("DEBUG") {
        Ok(s) => s == "true",
        Err(_) => false
    }
}
