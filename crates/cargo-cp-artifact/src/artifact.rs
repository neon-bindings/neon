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
