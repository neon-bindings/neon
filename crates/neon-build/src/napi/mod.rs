use std::path::PathBuf;

fn out_dir() -> PathBuf {
    std::env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .expect("Expected OUT_DIR environment variable")
}

fn output_file() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("Expected CARGO_MANIFEST_DIR environment variable")
        .join("index.node")
}

fn is_env(env_var: &str, value: &str) -> bool {
    std::env::var_os(env_var)
        .map(|v| v == value)
        .unwrap_or(false)
}

fn setup_unix() {
    println!("cargo:rustc-cdylib-link-arg=-o");
    println!("cargo:rustc-cdylib-link-arg={}", output_file().display());
}

fn setup_windows() {
    let pdb_file = out_dir().join("index.pdb");

    println!("cargo:rustc-cdylib-link-arg=/OUT:{}", output_file().display());
    println!("cargo:rustc-cdylib-link-arg=/PDB:{}", pdb_file.display());
}

pub(crate) fn setup() {
    let is_windows = is_env("CARGO_CFG_TARGET_OS", "windows");
    let is_gnu = is_env("CARGO_CFG_TARGET_ENV", "gnu");

    if is_windows && !is_gnu {
        setup_windows();
    } else {
        setup_unix();
    }
}
