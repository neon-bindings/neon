use std::ffi::OsStr;
use std::fs::{create_dir_all, remove_file};
use std::io::ErrorKind;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ArtifactKind {
    Bin,
    CDylib,
    Dylib,
}

impl ArtifactKind {
    pub fn parse(str: &impl AsRef<str>) -> Option<Self> {
        match str.as_ref() {
            "bin" => Some(ArtifactKind::Bin),
            "cdylib" => Some(ArtifactKind::CDylib),
            "dylib" => Some(ArtifactKind::Dylib),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Artifact {
    pub kind: ArtifactKind,
    pub crate_name: String,
}

fn is_newer(from: &Path, to: &Path) -> bool {
    if let (Ok(from_meta), Ok(to_meta)) = (from.metadata(), to.metadata()) {
        if let (Ok(from_mtime), Ok(to_mtime)) = (from_meta.modified(), to_meta.modified()) {
            return from_mtime > to_mtime;
        }
    }

    return true;
}

impl Artifact {

    // FIXME: return Result
    pub fn copy(&self, from: &Path, to: &Path) {
        if !is_newer(from, to) {
            return;
        }

        if let Some(basename) = to.parent() {
            // FIXME: panic
            create_dir_all(basename).expect("Couldn't create directories for output file");
        }

        // Apple Silicon (M1, etc.) requires shared libraries to be signed. However,
        // the macOS code signing cache isn't cleared when overwriting a file.
        // Deleting the file before copying works around the issue.
        //
        // Unfortunately, this workaround is incomplete because the file must be
        // deleted from the location it is loaded. If further steps in the user's
        // build process copy or move the file in place, the code signing cache
        // will not be cleared.
        //
        // https://github.com/neon-bindings/neon/issues/911
        if to.extension() == Some(OsStr::new("node")) {
            if let Err(err) = remove_file(to) {
                match err.kind() {
                    ErrorKind::NotFound => {}
                    // FIXME: panic
                    _ => { panic!("Couldn't overwrite {}", to.to_string_lossy()); }
                }
            }
        }

        // FIXME: panic
        std::fs::copy(from, to).expect("Couldn't copy file");
    }

}