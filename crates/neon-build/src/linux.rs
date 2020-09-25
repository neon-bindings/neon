use cfg_if::cfg_if;

pub fn setup() {
    cfg_if! {
        if #[cfg(feature = "output-lib")] {
            let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").unwrap();
            let output = std::path::Path::new(&manifest_dir).join("index.node");

            println!("cargo:rustc-cdylib-link-arg=-o");
            println!("cargo:rustc-cdylib-link-arg={}", output.display());
        }
    }
}
