extern crate regex;

use regex::Regex;
use std::env;
use std::process::Command;

#[cfg(unix)]
fn node_gyp() -> Command {
    Command::new("node-gyp")
}

#[cfg(windows)]
fn node_gyp() -> Command {
    let mut cmd = Command::new("cmd.exe");
    cmd.args(&["/C", "node-gyp"]);
    cmd
}

// The node-gyp output includes platform information in a string
// that looks like:
//
//     gyp info using node@8.3.0 | win32 | x64
fn parse_node_major_version(node_gyp_output: &str) -> u8 {
    let version_regex = Regex::new(
        r"node@(?P<version>\d+\.\d+\.\d+)\s+\|\s+(?P<platform>\w+)\s+\|\s(?P<arch>ia32|x64)",
    ).unwrap();

    version_regex
        .captures(&node_gyp_output)
        .and_then(|captures| captures.name("version"))
        .and_then(|version| version.as_str().split('.').next())
        .and_then(|major| major.parse().ok())
        .unwrap_or(0)
}

fn main() {
    let output = node_gyp()
        .args(&["list"])
        .output()
        .expect("Failed to run \"node-gyp list\" for neon!");

    let node_major_version = parse_node_major_version(&String::from_utf8_lossy(&output.stderr));
    if node_major_version >= 10 {
        println!("cargo:rustc_config=node_10")
    }

    if cfg!(windows) {
        println!(
            "cargo:node_root_dir={}",
            env::var("DEP_NEON_NODE_ROOT_DIR").unwrap()
        );
        println!(
            "cargo:node_arch={}",
            env::var("DEP_NEON_NODE_ARCH").unwrap()
        );
        println!(
            "cargo:node_lib_file={}",
            env::var("DEP_NEON_NODE_LIB_FILE").unwrap()
        );
    }

    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=neon_profile={:?}", profile);
    }
}
