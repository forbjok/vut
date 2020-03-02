use serde_derive::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
pub struct UpdateFilesDef {
    pub globs: Globs,
    pub updater: String,
    pub encoding: Option<String>,
}
