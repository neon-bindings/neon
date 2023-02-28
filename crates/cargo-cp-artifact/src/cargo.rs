use crate::artifact::{Artifact, ArtifactKind};

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{copy, create_dir_all, remove_file};
use std::io::ErrorKind;
use std::process::{Stdio, Command};
use std::path::{Path, PathBuf};
use cargo_metadata::{Message, Target};

#[derive(Debug, PartialEq, Eq)]
pub struct CargoCommand {
    pub artifacts: HashMap<Artifact, Vec<String>>,
    pub command: String,
    pub args: Vec<String>,
}

pub fn push_artifact(
    map: &mut HashMap<Artifact, Vec<String>>,
    kind: ArtifactKind,
    crate_name: String,
    output_file: String,
) {
    let key = Artifact { kind, crate_name };

    if !map.contains_key(&key) {
        let _ = map.insert(key, vec![output_file]);
    } else {
        map.get_mut(&key).unwrap().push(output_file);
    }
}

pub enum Status {
    Success,
    Failure,
}

fn is_newer(filename: &impl AsRef<Path>, output_file: &impl AsRef<Path>) -> bool {
    let filename = filename.as_ref();
    let output_file = output_file.as_ref();

    if let (Ok(meta1), Ok(meta2)) = (filename.metadata(), output_file.metadata()) {
        if let (Ok(mtime1), Ok(mtime2)) = (meta1.modified(), meta2.modified()) {
            return mtime1 > mtime2;
        }
    }

    return true;
}

// FIXME: return Result
fn copy_artifact(_artifact: &Artifact, filename: PathBuf, output_file: PathBuf) {
    if !is_newer(&filename, &output_file) {
        return;
    }

    if let Some(basename) = output_file.parent() {
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
    if output_file.extension() == Some(OsStr::new("node")) {
        if let Err(err) = remove_file(&output_file) {
            match err.kind() {
                ErrorKind::NotFound => {}
                // FIXME: panic
                _ => { panic!("Couldn't overwrite {}", output_file.to_string_lossy()); }
            }
        }
    }

    // FIXME: panic
    copy(&filename, &output_file).expect("Couldn't copy file");
}

impl CargoCommand {
    pub fn new(
        artifacts: HashMap<Artifact, Vec<String>>,
        command: String,
        args: Vec<String>,
    ) -> Self {
        Self { artifacts, command, args }
    }

    pub fn exec(self) -> Status {
        let mut command = Command::new(self.command)
            .args(&self.args)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(); // FIXME: unwrap

        let reader = std::io::BufReader::new(command.stdout.take().unwrap()); // FIXME: unwrap
        for message in cargo_metadata::Message::parse_stream(reader) {
            if let Message::CompilerArtifact(artifact) = message.unwrap() { // FIXME: unwrap
                let Target { kind: kinds, name, .. } = artifact.target;
                for (kind, filename) in kinds.iter().zip(artifact.filenames) {
                    if let Some(kind) = ArtifactKind::parse(kind) {
                        let crate_name = name.clone();
                        let artifact = Artifact { kind, crate_name };
                        if let Some(output_files) = self.artifacts.get(&artifact) {
                            for output_file in output_files {
                                copy_artifact(
                                    &artifact,
                                    filename.clone().into(),
                                    Path::new(output_file).to_path_buf(),
                                );
                            }
                        }
                    }
                }
            }
        }

        // FIXME: panic
        if command.wait().expect("Couldn't get cargo's exit status").success() {
            Status::Success
        } else {
            Status::Failure
        }
    }
}
