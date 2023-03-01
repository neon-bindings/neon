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

pub enum ArtifactError {
    MkdirFailed(std::io::Error),
    OverwriteFailed(std::io::Error),
    CopyFailed(std::io::Error),
}

impl std::fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactError::MkdirFailed(err) => {
                write!(f, "Could not create directory: {}", err)
            }
            ArtifactError::OverwriteFailed(err) => {
                write!(f, "Could not delete .node file: {}", err)
            }
            ArtifactError::CopyFailed(err) => {
                write!(f, "Could not copy artifact: {}", err)
            }
        }
    }
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
    pub fn copy(&self, from: &Path, to: &Path) -> Result<(), ArtifactError> {
        if !is_newer(from, to) {
            return Ok(());
        }

        if let Some(basename) = to.parent() {
            create_dir_all(basename).map_err(ArtifactError::MkdirFailed)?;
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
                    _ => {
                        return Err(ArtifactError::OverwriteFailed(err));
                    }
                }
            }
        }

        std::fs::copy(from, to).map_err(ArtifactError::CopyFailed)?;
        Ok(())
    }
}
