use serde_derive::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
pub struct UpdateFilesDef {
    pub globs: Globs,
    pub updater: String,
}

/// One or more file updater types
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum FileUpdaterTypes {
    Single(String),
    Multiple(HashSet<String>),
}
