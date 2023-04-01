use crate::artifact::ArtifactKind;
use crate::cargo::Status;
use crate::copy::{CopyMap, CopyPlan};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedArtifactKind(String),
    MissingArtifactKind,
    MissingCrateName,
    MissingOutputFile,
    EnvVarNotFound,
    UnexpectedOption(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedArtifactKind(found) => {
                write!(f, "Unexpected artifact type: {found}")
            }
            ParseError::MissingArtifactKind => {
                writeln!(f, "Missing artifact type.")?;
                writeln!(f, "")?;
                write!(f, "cargo-cp-artifact -a cdylib my-crate index.node ")?;
                write!(f, "-- cargo build --message-format=json-render-diagnostics")
            }
            ParseError::MissingCrateName => {
                writeln!(f, "Missing crate name.")?;
                writeln!(f, "")?;
                write!(f, "cargo-cp-artifact -a cdylib my-crate index.node ")?;
                write!(f, "-- cargo build --message-format=json-render-diagnostics")
            }
            ParseError::MissingOutputFile => {
                writeln!(f, "Missing output file.")?;
                writeln!(f, "")?;
                write!(f, "cargo-cp-artifact -a cdylib my-crate index.node ")?;
                write!(f, "-- cargo build --message-format=json-render-diagnostics")
            }
            ParseError::EnvVarNotFound => {
                write!(f, "Could not find the `{NPM_ENV}` environment variable. ")?;
                write!(f, "Expected to be executed from an `npm` command.")
            }
            ParseError::UnexpectedOption(found) => {
                write!(f, "Unexpected option: {found}")
            }
        }
    }
}

pub struct Args<T>(T);

impl Args<std::iter::Skip<std::env::Args>> {
    fn new(skip: usize) -> Self {
        Self(std::env::args().skip(skip))
    }
}

impl<T: Iterator<Item = String>> Args<T> {
    fn next(&mut self) -> Option<String> {
        let Self(args) = self;
        args.next()
    }

    fn rest(self) -> Vec<String> {
        let Self(args) = self;
        args.collect()
    }

    fn get_artifact_kind(&mut self, token: &str) -> Result<ArtifactKind, ParseError> {
        if token.len() == 3 && &token[1..2] != "-" {
            validate_artifact_kind(&token[2..3])
        } else {
            match self.next() {
                Some(kind) => validate_artifact_kind(kind.as_str()),
                None => Err(ParseError::MissingArtifactKind),
            }
        }
    }

    fn parse<F>(mut self, get_crate_name: F) -> Result<CopyPlan, ParseError>
    where
        F: Fn() -> Result<String, ParseError>,
    {
        let mut artifacts = CopyMap::new();

        loop {
            let token = match self.next() {
                Some(token) => token,
                None => { break; }
            };
            let token = token.as_str();

            if token == "--" {
                break;
            }

            if token == "--artifact" || (token.len() <= 3 && token.starts_with("-a")) {
                let kind = self.get_artifact_kind(token)?;
                let crate_name = self.next().ok_or(ParseError::MissingCrateName)?;
                let output_file = self.next().ok_or(ParseError::MissingOutputFile)?;
                artifacts.add(kind, crate_name, output_file);
                continue;
            }

            if token == "--npm" || (token.len() <= 3 && token.starts_with("-n")) {
                let kind = self.get_artifact_kind(token)?;
                let mut crate_name = get_crate_name()?;

                if let Some((left, right)) = crate_name.split_once('/') {
                    // This is a namespaced package; assume the crate is the un-namespaced version
                    if left.starts_with("@") {
                        crate_name = right.to_string();
                    }
                }

                let output_file = self.next().ok_or(ParseError::MissingOutputFile)?;
                artifacts.add(kind, crate_name, output_file);
                continue;
            }

            return Err(ParseError::UnexpectedOption(token.to_string()));
        }

        Ok(match self.next() {
            Some(command) => CopyPlan::cargo(artifacts, command, self.rest()),
            None => CopyPlan::stdin(artifacts)
        })
    }
}

fn validate_artifact_kind(kind: &str) -> Result<ArtifactKind, ParseError> {
    match kind {
        "b" | "bin" => Ok(ArtifactKind::Bin),
        "c" | "cdylib" => Ok(ArtifactKind::CDylib),
        "d" | "dylib" => Ok(ArtifactKind::Dylib),
        _ => Err(ParseError::UnexpectedArtifactKind(kind.to_string())),
    }
}

const NPM_ENV: &'static str = "npm_package_name";

fn get_crate_name_from_env() -> Result<String, ParseError> {
    std::env::var(NPM_ENV).or(Err(ParseError::EnvVarNotFound))
}

pub fn run(skip: usize) -> Status {
    let cargo = match Args::new(skip).parse(get_crate_name_from_env) {
        Ok(cargo) => cargo,
        Err(err) => {
            eprintln!("{err}");
            return Status::Failure;
        }
    };

    if let Err(err) = cargo.exec() {
        eprintln!("{err}");
        Status::Failure
    } else {
        Status::Success
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::artifact::{Artifact, ArtifactKind};
    use crate::cargo::CargoCommand;
    use crate::copy::{CopyAction, CopyPlan};

    impl<'a> Args<std::vec::IntoIter<String>> {
        fn from_vec(v: Vec<String>) -> Self {
            Self(v.into_iter())
        }
    }

    macro_rules! args {
        [$($s:literal),*] => {
            Args::from_vec(vec![$($s.to_string()),*])
        }
    }

    macro_rules! assert_err {
        ($actual:expr, $expected:expr, $($arg:tt)+) => {
            {
                match $actual {
                    Ok(_) => { panic!($($arg)+); }
                    Err(error) => {
                        assert_eq!(error, $expected, $($arg)+);
                    }
                }
            }
        }
    }

    fn get_crate_name_ok() -> Result<String, ParseError> {
        Ok("my-crate".to_string())
    }

    fn get_crate_name_with_namespace() -> Result<String, ParseError> {
        Ok("@my-namespace/my-crate".to_string())
    }

    fn get_crate_name_err() -> Result<String, ParseError> {
        Err(ParseError::EnvVarNotFound)
    }

    #[test]
    fn test_invalid_artifact_type() {
        let r = args!["-an", "a", "b", "--"].parse(get_crate_name_ok);
        assert_err!(
            r,
            ParseError::UnexpectedArtifactKind("n".to_string()),
            "expected artifact type parse error",
        );
    }

    #[test]
    fn test_missing_env_var() {
        let r = args!["-nc", "a", "b", "--"].parse(get_crate_name_err);
        assert_err!(r, ParseError::EnvVarNotFound, "expected env var error");
    }

    #[test]
    fn test_missing_command() {
        let cmd = args!["-ac", "my-crate", "my-bin"]
            .parse(get_crate_name_ok)
            .expect("expected successful parse");

        assert_eq!(
            cmd,
            example_stdin(),
            "expected stdin plan: {:?}",
            cmd
        );

        let cmd = args!["-ac", "my-crate", "my-bin", "--"]
            .parse(get_crate_name_ok)
            .expect("expected successful parse");
        assert_eq!(
            cmd,
            example_stdin(),
            "expected stdin plan: {:?}",
            cmd
        );
    }

    #[test]
    fn test_invalid_option() {
        let r = args!["-q"].parse(get_crate_name_ok);
        assert_err!(
            r,
            ParseError::UnexpectedOption("-q".to_string()),
            "expected bad option error"
        );
    }

    fn example_artifact1() -> Artifact {
        Artifact {
            kind: ArtifactKind::Bin,
            crate_name: "my-crate".to_string(),
        }
    }

    fn example_artifact2() -> Artifact {
        Artifact {
            kind: ArtifactKind::Dylib,
            crate_name: "a".to_string(),
        }
    }

    fn example_artifact3() -> Artifact {
        Artifact {
            kind: ArtifactKind::CDylib,
            crate_name: "my-crate".to_string(),
        }
    }

    fn example_stdin() -> CopyPlan {
        let mut artifacts = CopyMap::new();
        let artifact = example_artifact3();
        artifacts.set(artifact, &["my-bin"]);

        let action = CopyAction::Stdin;

        CopyPlan { artifacts, action }
    }

    fn example_cargo_command() -> CopyPlan {
        let mut artifacts = CopyMap::new();
        let artifact = example_artifact1();
        artifacts.set(artifact, &["my-bin"]);

        let command = "a".to_string();
        let args = vec!["b".to_string(), "c".to_string()];

        let action = CopyAction::Cargo(CargoCommand {
            command,
            args,
        });

        CopyPlan { artifacts, action }
    }

    fn example_complex_cargo_command() -> CopyPlan {
        let mut artifacts = CopyMap::new();

        artifacts.set(example_artifact1(), &["my-bin", "other-copy"]);
        artifacts.set(example_artifact2(), &["b"]);
        artifacts.set(example_artifact3(), &["index.node"]);

        let command = "a".to_string();
        let args = vec!["b".to_string(), "c".to_string()];

        let action = CopyAction::Cargo(CargoCommand {
            command,
            args,
        });

        CopyPlan { artifacts, action }
    }

    #[test]
    fn test_artifact_option() {
        let cmd = args![
            "--artifact",
            "bin",
            "my-crate",
            "my-bin",
            "--",
            "a",
            "b",
            "c"
        ]
        .parse(get_crate_name_ok)
        .expect("expected successful parse");

        assert_eq!(cmd, example_cargo_command(), "improperly parsed: {:?}", cmd);

        let cmd = args!["-a", "bin", "my-crate", "my-bin", "--", "a", "b", "c"]
            .parse(get_crate_name_ok)
            .expect("expected successful parse");

        assert_eq!(cmd, example_cargo_command(), "improperly parsed: {:?}", cmd);

        let cmd = args!["-ab", "my-crate", "my-bin", "--", "a", "b", "c"]
            .parse(get_crate_name_ok)
            .expect("expected successful parse");

        assert_eq!(cmd, example_cargo_command(), "improperly parsed: {:?}", cmd);
    }

    #[test]
    fn test_npm_option() {
        let cmd = args!["--npm", "bin", "my-bin", "--", "a", "b", "c"]
            .parse(get_crate_name_ok)
            .expect("expected successful parse");

        assert_eq!(cmd, example_cargo_command(), "improperly parsed: {:?}", cmd);

        let cmd = args!["-n", "bin", "my-bin", "--", "a", "b", "c"]
            .parse(get_crate_name_ok)
            .expect("expected successful parse");

        assert_eq!(cmd, example_cargo_command(), "improperly parsed: {:?}", cmd);

        let cmd = args!["-nb", "my-bin", "--", "a", "b", "c"]
            .parse(get_crate_name_ok)
            .expect("expected successful parse");

        assert_eq!(cmd, example_cargo_command(), "improperly parsed: {:?}", cmd);
    }

    #[test]
    fn test_namespace_removal() {
        let cmd = args!["--npm", "bin", "my-bin", "--", "a", "b", "c"]
            .parse(get_crate_name_with_namespace)
            .expect("expected successful parse");

        assert_eq!(cmd, example_cargo_command(), "improperly parsed: {:?}", cmd);
    }

    #[test]
    fn test_complex_command() {
        let cmd: CopyPlan = args![
            "-nb",
            "my-bin",
            "--artifact",
            "d",
            "a",
            "b",
            "-ac",
            "my-crate",
            "index.node",
            "--npm",
            "bin",
            "other-copy",
            "--",
            "a",
            "b",
            "c"
        ]
        .parse(get_crate_name_ok)
        .expect("expected successful parse");

        assert_eq!(
            cmd,
            example_complex_cargo_command(),
            "improperly parsed: {:?}",
            cmd
        );
    }
}