use crate::artifact::{Artifact, ArtifactError, ArtifactKind};
use crate::cargo::{CargoCommand, CargoError, CargoStream};

use cargo_metadata::{Message, Target, MessageIter};
use std::collections::HashMap;
use std::io::StdinLock;
use std::path::Path;
use std::process::Child;

pub enum CopyError {
    CargoFailed(CargoError),
    ChildFailed(std::io::Error),
    ArtifactCopyFailed(ArtifactError),
}

impl From<CargoError> for CopyError {
    fn from(value: CargoError) -> Self {
        CopyError::CargoFailed(value)
    }
}

impl std::fmt::Display for CopyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CopyError::CargoFailed(err) => {
                err.fmt(f)
            }
            CopyError::ChildFailed(err) => {
                write!(f, "Command failed to exit: {}", err)
            }
            CopyError::ArtifactCopyFailed(err) => {
                write!(f, "Failed to copy artifact: {}", err)
            }
        }
    }
}

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

    pub fn copy(&self, artifact: &Artifact, from: &Path) -> Result<(), CopyError> {
        if let Some(output_files) = self.0.get(&artifact) {
            for output_file in output_files {
                artifact
                    .copy(from, Path::new(output_file))
                    .map_err(CopyError::ArtifactCopyFailed)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
impl CopyMap {
    pub fn set(&mut self, artifact: Artifact, output_files: &[&str]) {
        let _ = self.0.insert(
            artifact,
            output_files.iter().map(|s| s.to_string()).collect(),
        );
    }
}

type StdinStream = MessageIter<StdinLock<'static>>;

fn stdin_stream() -> StdinStream {
    Message::parse_stream(std::io::stdin().lock())
}

#[derive(Debug, PartialEq, Eq)]
pub enum CopyAction {
    Cargo(CargoCommand),
    Stdin,
}

impl CopyAction {
    fn start(self) -> Result<(CopyStream, Option<Child>), CopyError> {
        match self {
            CopyAction::Cargo(command) => {
                let (child, stream) = command.spawn()?;
                Ok((CopyStream::Cargo(stream), Some(child)))
            }
            CopyAction::Stdin => {
                Ok((CopyStream::Stdin(stdin_stream()), None))
            }
        }
    }
}
enum CopyStream {
    Cargo(CargoStream),
    Stdin(StdinStream),
}

impl Iterator for CopyStream {
    type Item = Result<Message, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CopyStream::Cargo(stream) => stream.next(),
            CopyStream::Stdin(stream) => stream.next(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CopyPlan {
    pub artifacts: CopyMap,
    pub action: CopyAction,
}

impl CopyPlan {
    pub fn cargo(artifacts: CopyMap, command: String, args: Vec<String>) -> Self {
        let action = CopyAction::Cargo(CargoCommand::new(command, args));
        Self { artifacts, action }
    }

    pub fn stdin(artifacts: CopyMap) -> Self {
        let action = CopyAction::Stdin;
        Self { artifacts, action }
    }

    pub fn exec(self) -> Result<(), CopyError> {
        let (stream, child) = self.action.start()?;

        for message in stream {
            let message = message.map_err(CargoError::MessageParseFailed)?;
            if let Message::CompilerArtifact(artifact) = message {
                let Target {
                    kind: kinds, name, ..
                } = artifact.target;
                for (kind, filename) in kinds.iter().zip(artifact.filenames) {
                    let from = filename.into_std_path_buf();
                    if let Some(kind) = ArtifactKind::parse(kind) {
                        let crate_name = name.clone();
                        let artifact = Artifact { kind, crate_name };
                        self.artifacts.copy(&artifact, &from)?;
                    }
                }
            }
        }

        if let Some(mut child) = child {
            child.wait().map_err(CopyError::ChildFailed)?;
        }

        Ok(())
    }
}
