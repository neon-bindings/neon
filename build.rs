use std::env;
use std::process::Command;

#[cfg(unix)]
fn node() -> Command {
    Command::new("node")
}

#[cfg(windows)]
fn node() -> Command {
    let mut cmd = Command::new("cmd.exe");
    cmd.args(&["/C", "node"]);
    cmd
}

// `node --version` outputs semver versions
fn parse_node_major_version(node_output: &str) -> u8 {
    // Discard the `v` prefix
    node_output
        .chars()
        .nth(2)
        .and_then(|major_version_string| major_version_string.to_digit(10))
        .map(|v| v as u8)
        .unwrap_or(0u8)
}

fn main() {
    let output = node()
        .args(&["--version"])
        .output()
        .expect("Failed to run \"node --version\" for neon!");

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
