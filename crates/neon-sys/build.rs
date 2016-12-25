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

#[cfg(unix)]
fn object_path(libname: &str) -> String {
    format!("build/{}/obj.target/{}/src/{}.o", mode(), libname, libname)
}

#[cfg(windows)]
fn object_path(libname: &str) -> String {
    format!("build\\{}\\obj\\{}\\{}.obj", mode(), libname, libname)
}

#[cfg(unix)]
const NPM_COMMAND : &'static str = "npm";

#[cfg(windows)]
const NPM_COMMAND : &'static str = "npm.cmd";

fn build_object_file() {
    // Ensure that all package.json dependencies and dev dependencies are installed.
    Command::new(NPM_COMMAND).arg("install").status().ok()
        .expect(r#"failed to run "npm install" for neon-sys"#);

    // Run the package.json `configure` script, which invokes `node-gyp configure` from the local node_modules.
    Command::new(NPM_COMMAND).arg("run").arg(if debug() { "configure-debug" } else { "configure-release" }).status().ok().unwrap();

    // Run the package.json `build` script, which invokes `node-gyp build` from the local node_modules.
    Command::new(NPM_COMMAND).arg("run").arg(if debug() { "build-debug" } else { "build-release" }).status().ok().unwrap();
}

fn link_library() {
    // Link the built object file into a static library.
    gcc::Config::new()
        .object(object_path("neon"))
        .compile("libneon.a");
}

fn main() {
    // 1. Build the object file from source using node-gyp.
    build_object_file();

    // 2. Link the library from the object file using gcc.
    link_library();
}
