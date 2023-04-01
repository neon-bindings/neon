use cargo_metadata::{Message, MessageIter};
use std::io::BufReader;
use std::process::{Command, Stdio, Child, ChildStdout};

pub enum CargoError {
    SpawnFailed(std::io::Error),
    MessageParseFailed(std::io::Error),
}

impl std::fmt::Display for CargoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CargoError::SpawnFailed(err) => {
                write!(f, "Could not execute command: {}", err)
            }
            CargoError::MessageParseFailed(err) => {
                write!(f, "Could not read command output: {}", err)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CargoCommand {
    pub command: String,
    pub args: Vec<String>,
}

pub enum Status {
    Success,
    Failure,
}

pub type CargoStream = MessageIter<BufReader<ChildStdout>>;

impl CargoCommand {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self { command, args }
    }

    pub fn spawn(self) -> Result<(Child, CargoStream), CargoError> {
        let mut child = Command::new(self.command)
            .args(&self.args)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(CargoError::SpawnFailed)?;

        let reader = BufReader::new(child.stdout.take().unwrap());
        let stream = Message::parse_stream(reader);
        Ok((child, stream))
    }
}
