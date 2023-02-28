use crate::artifact::{Artifact, ArtifactKind};

use std::collections::HashMap;
use std::process::{Stdio, Command};
use std::path::Path;
use cargo_metadata::{Message, Target};

#[derive(Debug, PartialEq, Eq)]
pub struct CopyMap(HashMap<Artifact, Vec<String>>);

impl CopyMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add(&mut self, kind: ArtifactKind, crate_name: String, output_file: String) {
        let key = Artifact { kind, crate_name };

        if !self.0.contains_key(&key) {
            let _ = self.0.insert(key, vec![output_file]);
        } else {
            self.0.get_mut(&key).unwrap().push(output_file);
        }
    }

    pub fn copy(&self, artifact: &Artifact, from: &Path) {
        if let Some(output_files) = self.0.get(&artifact) {
            for output_file in output_files {
                artifact.copy(from, Path::new(output_file));
            }
        }
    }
}

#[cfg(test)]
impl CopyMap {
    pub fn set(&mut self, artifact: Artifact, output_files: &[&str]) {
        let _ = self.0.insert(
            artifact,
            output_files
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CargoCommand {
    pub artifacts: CopyMap,
    pub command: String,
    pub args: Vec<String>,
}

pub enum Status {
    Success,
    Failure,
}

impl CargoCommand {
    pub fn new(artifacts: CopyMap, command: String, args: Vec<String>) -> Self {
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
                    let from = filename.into_std_path_buf();
                    if let Some(kind) = ArtifactKind::parse(kind) {
                        let crate_name = name.clone();
                        let artifact = Artifact { kind, crate_name };
                        self.artifacts.copy(&artifact, &from);
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
