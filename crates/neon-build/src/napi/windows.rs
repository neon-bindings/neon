use std::io::{Error, ErrorKind, Read, Result, Write};
use std::path::Path;
use std::process::Command;

fn node_version() -> Result<String> {
    let output = Command::new("node").arg("-v").output()?;
    if !output.status.success() {
        let _ = std::io::stderr().write_all(&output.stderr);
        panic!("Could not download node.lib. There is likely more information from stderr above.");
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
    // npm_config_target should not contain a leading "v"
    Ok(stdout.trim().trim_start_matches('v').to_string())
}

fn download_node_lib(version: &str, arch: &str) -> Result<Vec<u8>> {
    // Assume we're building for node if a disturl is not specified.
    let dist_url = std::env::var("NPM_CONFIG_DISTURL").unwrap_or("https://nodejs.org/dist".into());
    let mut request = ureq::get(&format!(
        "{dist_url}/v{version}/win-{arch}/node.lib",
        dist_url = dist_url,
        version = version,
        arch = arch,
    ));
    let response = request.call();
    let mut bytes = vec![];
    response.into_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Set up the build environment by setting Cargo configuration variables.
pub(crate) fn setup() {
    // If the user specified a node.lib path, we do not need to download
    if let Some(node_lib_path) = std::env::var_os("NEON_NODE_LIB") {
        let node_lib_path = Path::new(&node_lib_path);
        // Clearing the file name returns the root+directory name
        let dir = node_lib_path.with_file_name("");
        let basename = node_lib_path.file_stem().expect("Could not parse lib name from NEON_NODE_LIB. Does the path include the full file name?");

        println!("cargo:rustc-link-search=native={}", dir.display());
        // `basename` is an OsStr, we can output it anyway by re-wrapping it in a Path
        // Both `dir` and `basename` will be mangled (contain replacement characters) if
        // they are not UTF-8 paths. If we don't mangle them though, Cargo will: so
        // non-UTF-8 paths are simply not supported.
        println!("cargo:rustc-link-lib={}", Path::new(basename).display());
        return;
    }

    let version = std::env::var("npm_config_target")
        .or_else(|_| node_version())
        .expect("Could not determine Node.js version");
    let arch = if std::env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "x86" {
        "x86"
    } else {
        "x64"
    };

    let node_lib_store_path = format!(r"{}/node-{}.lib", env!("OUT_DIR"), arch);

    // Download node.lib if it does not exist
    if let Err(_) = std::fs::metadata(&node_lib_store_path) {
        let node_lib = download_node_lib(&version, arch).expect("Could not download `node.lib`");
        std::fs::write(&node_lib_store_path, &node_lib).expect("Could not save `node.lib`");
    }

    println!("cargo:rustc-link-search=native={}", env!("OUT_DIR"));
    println!("cargo:rustc-link-lib=node-{}", arch);
    println!("cargo:rustc-cdylib-link-arg=delayimp.lib");
    println!("cargo:rustc-cdylib-link-arg=/DELAYLOAD:node.exe");
}
