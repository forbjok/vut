use serde_derive::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VersionSourceDef {
    Simple(Patterns),
    Detailed(VersionSourceDetail),
}

#[derive(Clone, Debug, Deserialize)]
pub struct VersionSourceDetail {
    pub pattern: Patterns,
    pub exclude_pattern: Option<Patterns>,
    pub types: Option<VersionSourceTypes>,
}
