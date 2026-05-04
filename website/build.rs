use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let out_path = PathBuf::from(out_dir).join("doctests.rs");
    fs::write(&out_path, "// generated\n").expect("write doctests.rs");
}
