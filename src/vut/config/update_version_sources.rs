use serde_derive::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VersionSourceDef {
    Simple(Globs),
    Detailed(VersionSourceDetail),
}

#[derive(Clone, Debug, Deserialize)]
pub struct VersionSourceDetail {
    pub globs: Globs,
    pub exclude_globs: Option<Globs>,
    pub types: Option<VersionSourceTypes>,
}
