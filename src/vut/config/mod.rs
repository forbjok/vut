use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde_derive::Deserialize;
use toml;

use crate::util;
use crate::vut::VutError;

mod custom_file_updater;
mod custom_source_type;
mod glob;
mod regex;
mod template_processor;
mod templates;
mod update_files;
mod update_version_sources;

pub use self::custom_file_updater::*;
pub use self::custom_source_type::*;
pub use self::glob::*;
pub use self::regex::*;
pub use self::template_processor::*;
pub use self::templates::*;
pub use self::update_files::*;
pub use self::update_version_sources::*;

pub const VUT_CONFIG_DEFAULT: &str = include_str!("default_config.toml");
pub const VUT_CONFIG_EXAMPLE: &str = include_str!("example_config.toml");

/// One or more version source types
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct VersionSourceTypes(pub HashSet<String>);

#[derive(Debug, Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct General {
    pub ignore: Globs,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AuthoritativeVersionSource {
    pub path: PathBuf,

    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct VutConfig {
    pub general: General,
    pub authoritative_version_source: Option<AuthoritativeVersionSource>,
    pub file_updaters: HashMap<String, CustomFileUpdaterTypeDef>,
    pub version_source_types: HashMap<String, CustomSourceTypeDef>,
    pub update_files: Vec<UpdateFilesDef>,
    pub update_version_sources: Vec<UpdateVersionSourcesDef>,
    pub templates: Vec<TemplatesDef>,
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
            ignore: Globs::Single("**/.git".to_owned()),
        }
    }
}

impl Default for VutConfig {
    fn default() -> Self {
        Self {
            general: General::default(),
            authoritative_version_source: None,
            file_updaters: HashMap::new(),
            version_source_types: HashMap::new(),
            update_files: Vec::new(),
            update_version_sources: Vec::new(),
            templates: vec![TemplatesDef {
                globs: Globs::Single("**/*.vutemplate".to_owned()),
                start_path: None,
                output_path: None,
                processor: None,
                encoding: None,
            }],
        }
    }
}

pub fn create_config_file(path: &Path, text: &str) -> Result<VutConfig, VutError> {
    util::create_file(&path)
        .map_err(|err| VutError::OpenConfig(err))?
        .write(text.as_bytes())
        .map_err(|err| VutError::WriteConfig(err))?;

    VutConfig::from_str(text)
}
