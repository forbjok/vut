use std::borrow::Cow;
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_derive::Deserialize;
use toml;

use crate::util;
use crate::vut::VutError;

/// One or more glob patterns.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Patterns {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub struct Template {
    pub pattern: Patterns,
    pub start_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub processor: Option<String>,
    pub encoding: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSourceDetail {
    pub path: String,
    pub types: HashSet<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum UpdateSource {
    Simple(String),
    Detailed(UpdateSourceDetail),
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct VutConfig {
    pub ignore: Vec<String>,
    pub update_sources: Vec<UpdateSource>,
    pub exclude_sources: Vec<String>,
    pub template: Vec<Template>,
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
            exclude_sources: Vec::new(),
            template: vec![Template {
                pattern: Patterns::Single("**/*.vutemplate".to_owned()),
                start_path: None,
                output_path: None,
                processor: None,
                encoding: None,
            }],
        }
    }
}
