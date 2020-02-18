use std::borrow::Cow;
use std::io::Read;
use std::path::Path;

use serde_derive::Deserialize;
use toml;

use crate::util;
use crate::vut::VutError;

#[derive(Debug, Deserialize)]
pub struct VutConfig {
    pub update_nested_sources: bool,
}

impl VutConfig {
    pub fn from_file(path: &Path) -> Result<Self, VutError> {
        let mut file = util::open_file(path).map_err(|err| VutError::OpenConfig(err))?;

        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)
            .map_err(|err| VutError::ReadConfig(err))?;

        let config: VutConfig =
            toml::from_str(&toml_str).map_err(|err| VutError::ParseConfig(Cow::Owned(err.to_string())))?;

        Ok(config)
    }
}

impl Default for VutConfig {
    fn default() -> Self {
        Self {
            update_nested_sources: false,
        }
    }
}
