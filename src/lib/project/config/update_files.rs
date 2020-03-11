use serde_derive::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UpdateFilesDef {
    pub globs: Globs,
    pub updater: String,
    pub encoding: Option<String>,
}
