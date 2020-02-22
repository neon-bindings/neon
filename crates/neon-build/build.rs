extern crate cfg_if;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(windows, feature = "neon-sys"))] {
        use std::env;
        use std::fs::File;
        use std::io::Write;
        use std::path::Path;

        // Extract linker metadata from neon-sys and save it in a text file.
        // The neon-build lib.rs will textually include them into constants.
        fn save(var: &str, filename: &str) {
            let path = Path::new(&env::var("OUT_DIR").unwrap()).join(filename);
            let mut buffer = File::create(path).unwrap();
            write!(buffer, "{}", env::var(var).unwrap()).unwrap();
        }

        fn main() {
            save("DEP_NEON_NODE_ROOT_DIR", "node_root_dir");
            save("DEP_NEON_NODE_ARCH", "node_arch");
            save("DEP_NEON_NODE_LIB_FILE", "node_lib_file");
        }
    } else {
        fn main() { }
    }
}
