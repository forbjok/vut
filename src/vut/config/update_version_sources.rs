use serde_derive::Deserialize;

use super::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UpdateVersionSourcesDef {
    pub globs: Globs,
    pub exclude_globs: Option<Globs>,
    pub types: Option<VersionSourceTypes>,
}

impl Default for UpdateVersionSourcesDef {
    fn default() -> Self {
        Self {
            globs: Globs(Vec::new()),
            exclude_globs: None,
            types: None,
        }
    }
}
