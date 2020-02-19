use std::borrow::Cow;
use std::io::Read;
use std::path::Path;

use serde_derive::Deserialize;
use toml;

use crate::util;
use crate::vut::VutError;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct VutConfig {
    pub ignore: Vec<String>,
    pub update_sources: Vec<String>,
}

impl VutConfig {
    pub fn from_str(s: &str) -> Result<Self, VutError> {
        let config: VutConfig = toml::from_str(s).map_err(|err| VutError::ParseConfig(Cow::Owned(err.to_string())))?;

        Ok(config)
    }

    pub fn from_file(path: &Path) -> Result<Self, VutError> {
        let mut file = util::open_file(path).map_err(|err| VutError::OpenConfig(err))?;

        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)
            .map_err(|err| VutError::ReadConfig(err))?;

        Self::from_str(&toml_str)
    }
}

impl Default for VutConfig {
    fn default() -> Self {
        Self {
            ignore: vec!["**/.git".to_owned()],
            update_sources: Vec::new(),
        }
    }
}
