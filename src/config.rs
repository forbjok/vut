use std::borrow::Cow;
use std::path::Path;

use serde_derive::Deserialize;
use serde_json;

use crate::util;
use crate::vut::VutError;

#[derive(Debug, Deserialize)]
pub struct VutConfig {
}

impl VutConfig {
    pub fn from_file(path: &Path) -> Result<Self, VutError> {
        let file = util::open_file(path)
            .map_err(|err| VutError::OpenConfig(err))?;

        let config: VutConfig = serde_json::from_reader(file)
            .map_err(|err| VutError::ParseConfig(Cow::Owned(err.to_string())))?;

        Ok(config)
    }
}

impl Default for VutConfig {
    fn default() -> Self {
        Self { }
    }
}
