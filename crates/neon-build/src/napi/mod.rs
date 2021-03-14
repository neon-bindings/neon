use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("Expected CARGO_MANIFEST_DIR environment variable")
}

fn out_dir() -> PathBuf {
    std::env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .expect("Expected OUT_DIR environment variable")
}

fn is_env(env_var: &str, value: &str) -> bool {
    std::env::var_os(env_var)
        .map(|v| v == value)
        .unwrap_or(false)
}

fn setup_unix(output_file: PathBuf) {
    println!("cargo:rustc-cdylib-link-arg=-o");
    println!("cargo:rustc-cdylib-link-arg={}", output_file.display());
}

fn setup_windows(output_file: PathBuf) {
    let pdb_file = output_file
        .file_name()
        .map(|file| out_dir().join(Path::new(file).with_extension("pdb")))
        .expect("Expected a neon output file name");

    println!("cargo:rustc-cdylib-link-arg=/OUT:{}", output_file.display());
    println!("cargo:rustc-cdylib-link-arg=/PDB:{}", pdb_file.display());
}

/// `Setup` acts as a builder for initializing a Neon build script
///
/// A default setup builder is provided as [`neon_build::setup()`](crate::setup()).
///
/// # Example
///
/// Output the neon module at `lib/native.node`
///
/// ```
/// # #[allow(clippy::needless_doctest_main)]
/// fn main() {
///     neon_build::Setup::options()
///         .output_dir("lib")
///         .output_dir("native.node")
///         .setup();
/// }
#[derive(Debug, Default)]
pub struct Setup {
    output_dir: Option<PathBuf>,
    output_file: Option<PathBuf>,
}

impl Setup {
    /// Create a new builder for Setup options
    pub fn options() -> Self {
        Default::default()
    }

    /// Sets the output directory for the native library.
    /// Defaults to the cargo manifest directory. If not absolute, paths will
    /// be relative to the cargo manifest directory.
    pub fn output_dir(&mut self, output_dir: impl AsRef<Path>) -> &mut Self {
        self.output_dir = Some(output_dir.as_ref().to_path_buf());
        self
    }

    /// Sets the name of the native library. Defaults to `index.node`. If not
    /// absolute, paths will be relative to the [`Setup::output_dir`].
    ///
    /// **Note**: Node.js requires that native libraries have the `.node`
    /// extension to be loaded by `require`.
    pub fn output_file(&mut self, output_file: impl AsRef<Path>) -> &mut Self {
        self.output_file = Some(output_file.as_ref().to_path_buf());
        self
    }

    /// Setup the Cargo build process for Neon. Should be called once from
    /// `fn main` in a cargo build script.
    pub fn setup(&self) {
        let output_file = self.absolute_output_file();
        let is_windows = is_env("CARGO_CFG_TARGET_OS", "windows");
        let is_gnu = is_env("CARGO_CFG_TARGET_ENV", "gnu");

        if is_windows && !is_gnu {
            setup_windows(output_file);
        } else {
            setup_unix(output_file);
        }
    }

    fn absolute_output_file(&self) -> PathBuf {
        let output_file = self
            .output_file
            .clone()
            .unwrap_or_else(|| PathBuf::from("index.node"));

        // Don't prepend `output_dir` if `output_file` is absolute
        if output_file.is_absolute() {
            return output_file;
        }

        let output_dir = if let Some(output_dir) = self.output_dir.clone() {
            // If `output_dir` is absolute, use it, otherwise
            // append it to `manifest_dir()`
            if output_dir.is_absolute() {
                output_dir
            } else {
                manifest_dir().join(output_dir)
            }
        } else {
            // Default to `manifest_dir()`
            manifest_dir()
        };

        output_dir.join(output_file)
    }
}

pub(crate) fn setup() {
    Setup::options().setup()
}

#[test]
fn test_absolute_output_file_defaults() {
    let expected = manifest_dir().join("index.node");
    let actual = Setup::options().absolute_output_file();

    assert_eq!(actual, expected);
}

#[test]
fn test_absolute_output_file_absolute_file() {
    let expected = PathBuf::from("/tmp/hello.node");
    let actual = Setup::options()
        .output_dir("/tmp/ignore/me")
        .output_file("/tmp/hello.node")
        .absolute_output_file();

    assert_eq!(actual, expected);
}

#[test]
fn test_absolute_output_file_absolute_dir() {
    let expected = PathBuf::from("/tmp/index.node");
    let actual = Setup::options().output_dir("/tmp").absolute_output_file();

    assert_eq!(actual, expected);
}

#[test]
fn test_absolute_output_file_relative_dir() {
    let expected = manifest_dir().join("lib").join("index.node");
    let actual = Setup::options().output_dir("lib").absolute_output_file();

    assert_eq!(actual, expected);
}

#[test]
fn test_absolute_output_file_relative_file() {
    let expected = manifest_dir().join("lib.node");
    let actual = Setup::options()
        .output_file("lib.node")
        .absolute_output_file();

    assert_eq!(actual, expected);
}
