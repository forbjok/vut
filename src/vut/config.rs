use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_derive::Deserialize;
use toml;

use crate::util;
use crate::vut::VutError;

pub const VUT_CONFIG_DEFAULT: &str = include_str!("default_config.toml");

/// One or more glob patterns.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Patterns {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub struct RegexCustomSourceType {
    pub file_name: String,
    pub regex: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum CustomSourceType {
    Regex(RegexCustomSourceType),
}

/// One or more version source types
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VersionSourceTypes {
    Single(String),
    Multiple(HashSet<String>),
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
pub struct VersionSourceDetail {
    pub pattern: Patterns,
    pub exclude_pattern: Option<Patterns>,
    pub types: Option<VersionSourceTypes>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VersionSource {
    Simple(Patterns),
    Detailed(VersionSourceDetail),
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct General {
    pub ignore: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthoritativeVersionSource {
    pub path: PathBuf,

    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct VutConfig {
    pub general: General,
    pub authoritative_version_source: Option<AuthoritativeVersionSource>,
    pub custom_source_types: HashMap<String, CustomSourceType>,
    pub version_source: Vec<VersionSource>,
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

impl Default for General {
    fn default() -> Self {
        Self {
            ignore: vec!["**/.git".to_owned()],
        }
    }
}

impl Default for VutConfig {
    fn default() -> Self {
        Self {
            general: General::default(),
            authoritative_version_source: None,
            custom_source_types: HashMap::new(),
            version_source: Vec::new(),
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

pub fn create_default_config_file(path: &Path) -> Result<VutConfig, VutError> {
    let default_config = VUT_CONFIG_DEFAULT.trim();

    util::create_file(&path)
        .map_err(|err| VutError::OpenConfig(err))?
        .write(default_config.as_bytes())
        .map_err(|err| VutError::WriteConfig(err))?;

    VutConfig::from_str(default_config)
}