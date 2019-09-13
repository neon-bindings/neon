extern crate cc;
extern crate regex;

use std::process::Command;
use std::env;
use std::fs;
use std::path::Path;
use regex::Regex;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let out_dir = Path::new(&out_dir);
    let native_from = Path::new(&crate_dir).join("native");
    let native_to = out_dir.join("native");

    // 1. Copy the native runtime library into the build directory.
    copy_native_library(&native_from, &native_to);

    // 2. Build the object file from source using node-gyp.
    build_object_file(&native_to);

    // 3. Link the library from the object file using gcc.
    link_library(&native_to);

    // 4. Copy native build artifacts
    copy_build_artifacts(&native_to, &out_dir);
}

fn copy_files(dir_from: impl AsRef<Path>, dir_to: impl AsRef<Path>) {
    for entry in fs::read_dir(dir_from.as_ref()).unwrap() {
        let entry = entry.unwrap();

        if entry.file_type().unwrap().is_dir() {
            continue;
        }

        let file_name = entry.file_name();

        let from = dir_from.as_ref().join(&file_name);
        let to = dir_to.as_ref().join(&file_name);

        fs::copy(from, to).unwrap();
    }
}

fn copy_native_library(native_from: impl AsRef<Path>, native_to: impl AsRef<Path>) {
    let native_from = native_from.as_ref();
    let native_to = native_to.as_ref();

    let src_from = native_from.join("src");
    let src_to = native_to.join("src");

    fs::create_dir_all(&src_to).unwrap();

    copy_files(&native_from, &native_to);
    copy_files(&src_from, &src_to);
}

#[cfg(unix)]
fn npm(cwd: &Path) -> Command {
    let mut cmd = Command::new("npm");
    cmd.current_dir(cwd);
    cmd
}

#[cfg(windows)]
fn npm(cwd: &Path) -> Command {
    let mut cmd = Command::new("cmd.exe");
    cmd.args(&["/C", "npm"]);
    cmd.current_dir(cwd);
    cmd
}

// The node-gyp output includes platform information in a string
// that looks like:
//
//     gyp info using node@8.3.0 | win32 | x64
fn parse_node_arch(node_gyp_output: &str) -> String {
    let version_regex = Regex::new(r"node@(?P<version>\d+\.\d+\.\d+)\s+\|\s+(?P<platform>\w+)\s+\|\s(?P<arch>ia32|x64)").unwrap();
    let captures = version_regex.captures(&node_gyp_output).unwrap();
    String::from(&captures["arch"])
}

// The node-gyp output includes the root directory of shared resources
// for the Node installation in a string that looks like:
//
//     '-Dnode_root_dir=C:\\Users\\dherman\\.node-gyp\\8.3.0'
fn parse_node_root_dir(node_gyp_output: &str) -> &str {
    let node_root_dir_flag_pattern = "'-Dnode_root_dir=";
    let node_root_dir_start_index = node_gyp_output
        .find(node_root_dir_flag_pattern)
        .map(|i| i + node_root_dir_flag_pattern.len())
        .expect("Couldn't find node_root_dir in node-gyp output.");
    let node_root_dir_end_index = node_gyp_output[node_root_dir_start_index..].find("'").unwrap() + node_root_dir_start_index;
    &node_gyp_output[node_root_dir_start_index..node_root_dir_end_index]
}

// The node-gyp output includes the name of the shared Node library file.
// In NPM versions <= v5.0.3, this was just the filename by itself, e.g.:
//
//     '-Dnode_lib_file=node.lib'
//
// In NPM versions >= v5.3.0, this was a templated absolute path with a
// reference to a gyp variable, e.g.:
//
//     '-Dnode_lib_file=C:\\Users\\dherman\\.node-gyp\\8.3.0\\<(target_arch)\\node.lib'
//
// Either way, we simply extract the value here. The `neon-build` crate
// processes it further.
fn parse_node_lib_file(node_gyp_output: &str) -> &str {
    let node_lib_file_flag_pattern = "'-Dnode_lib_file=";
    let node_lib_file_start_index = node_gyp_output
        .find(node_lib_file_flag_pattern)
        .map(|i| i + node_lib_file_flag_pattern.len())
        .expect("Couldn't find node_lib_file in node-gyp output.");
    let node_lib_file_end_index = node_gyp_output[node_lib_file_start_index..].find("'").unwrap() + node_lib_file_start_index;
    &node_gyp_output[node_lib_file_start_index..node_lib_file_end_index]
}

fn build_object_file(native_dir: &Path) {
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
    npm(native_dir).args(&["install", "--silent"]).status().ok().expect("Failed to run \"npm install\" for neon-runtime!");

    // Run `node-gyp configure` in verbose mode to read node_root_dir on Windows.
    let output = npm(native_dir)
        .args(&["run", if debug() { "configure-debug" } else { "configure-release" }])
        .output()
        .expect("Failed to run \"node-gyp configure\" for neon-runtime!");

    if !output.status.success() {
        panic!(format!(
            "Failed to run \"node-gyp configure\" for neon-runtime!\n Out: {}\n Err: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    if cfg!(windows) {
        let node_gyp_output = String::from_utf8_lossy(&output.stderr);
        println!("cargo:node_arch={}", parse_node_arch(&node_gyp_output));
        println!("cargo:node_root_dir={}", parse_node_root_dir(&node_gyp_output));
        println!("cargo:node_lib_file={}", parse_node_lib_file(&node_gyp_output));
    }

    // Run `node-gyp build`.
    npm(native_dir)
        .args(&["run", if debug() { "build-debug" } else { "build-release" }])
        .status()
        .ok()
        .expect("Failed to run \"node-gyp build\" for neon-runtime!");
}

// Link the built object file into a static library.
fn link_library(native_dir: &Path) {
    let configuration = if debug() { "Debug" } else { "Release" };

    let object_path = if cfg!(unix) {
        native_dir
            .join("build")
            .join(configuration)
            .join("obj.target")
            .join("neon")
            .join("src")
            .join("neon.o")
    } else {
        native_dir
            .join("build")
            .join(configuration)
            .join("obj")
            .join("neon")
            .join("neon.obj")
    };

    cc::Build::new().cpp(true).object(object_path).compile("libneon.a");
}

#[cfg(unix)]
fn copy_build_artifacts(_native_dir: &Path, _out_dir: &Path) {}

#[cfg(windows)]
fn copy_build_artifacts(native_dir: &Path, out_dir: &Path) {
    let configuration = if debug() { "Debug" } else { "Release" };
    let win_delay_file = "win_delay_load_hook.obj";
    let win_delay_dest = out_dir.join(win_delay_file);
    let win_delay_source = native_dir
        .join("build")
        .join(configuration)
        .join("obj")
        .join("neon")
        .join(win_delay_file);

    // Win delay hook is only needed for electron, warn instead of crash if not found
    if let Err(err) = fs::copy(win_delay_source, win_delay_dest) {
        if err.kind() == std::io::ErrorKind::NotFound {
            eprintln!("warning: {} could not be found", win_delay_file);
        } else {
            panic!("{:?}", err);
        }
    }
}

fn debug() -> bool {
    match env::var("DEBUG") {
        Ok(s) => s == "true",
        Err(_) => false
    }
}
