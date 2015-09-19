extern crate gcc;

use std::process::Command;
use std::env;

fn debug() -> bool {
    match env::var("DEBUG") {
        Ok(s) => s == "true",
        Err(_) => false
    }
}

fn mode() -> &'static str {
    if debug() { "Debug" } else { "Release" }
}

fn object_path(libname: &str) -> String {
    format!("build/{}/obj.target/{}/src/{}.o", mode(), libname, libname)
}

fn build_object_file() {
    // Ensure that all package.json dependencies and dev dependencies are installed.
    Command::new("npm").arg("install").status().ok().unwrap();

    // Run the package.json `configure` script, which invokes `node-gyp configure` from the local node_modules.
    let mut config_args = vec!["run", "configure"];
    if debug() {
        config_args.push("-d");
    }
    Command::new("npm").args(&config_args[..]).status().ok().unwrap();

    // Run the package.json `build` script, which invokes `node-gyp build` from the local node_modules.
    let mut build_args = vec!["run", "build"];
    if debug() {
        build_args.push("-d");
    }
    Command::new("npm").args(&build_args[..]).status().ok().unwrap();
}

fn link_library() {
    // Link the built object file into a static library.
    gcc::Config::new()
        .object(object_path("nanny"))
        .compile("libnanny.a");
}

fn main() {
    // 1. Build the object file from source using node-gyp.
    build_object_file();

    // 2. Link the library from the object file using gcc.
    link_library();
}
