use std::path::Path;

use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter, EnumString};

use crate::project::VutError;
use crate::util;
use crate::version::Version;

mod cargo;
mod custom_regex;
mod npm;
mod version_file;

pub use cargo::*;
pub use custom_regex::CustomRegexSourceTemplate;
pub use npm::*;
pub use version_file::*;

/// Trait representing the authoritative source of a project's version
pub trait VersionSource {
    fn get_path(&self) -> &Path;
    fn exists(&self) -> bool;
    fn get_version(&self) -> Result<Version, VutError>;
    fn set_version(&mut self, version: &Version) -> Result<(), VutError>;
}

#[derive(AsRefStr, Debug, Clone, EnumIter, EnumString, Eq, Hash, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum VersionSourceType {
    Vut,
    Cargo,
    Npm,
}

impl VersionSourceType {
    pub fn create_from_path(&self, path: &Path) -> Option<Box<dyn VersionSource>> {
        match self {
            Self::Vut => VersionFileSource::from_path(path).map(|vs| Box::new(vs) as Box<dyn VersionSource>),
            Self::Cargo => CargoSource::from_path(path).map(|vs| Box::new(vs) as Box<dyn VersionSource>),
            Self::Npm => NpmSource::from_path(path).map(|vs| Box::new(vs) as Box<dyn VersionSource>),
        }
    }
}

pub fn first_version_source_from_path(path: &Path) -> Option<(VersionSourceType, Box<dyn VersionSource>)> {
    for st in VersionSourceType::iter() {
        if let Some(source) = st.create_from_path(path) {
            return Some((st, source));
        }
    }

    None
}

pub fn locate_first_version_source_from(start_path: &Path) -> Option<Box<dyn VersionSource>> {
    util::find_outwards(start_path, |path| first_version_source_from_path(path)).map(|(_, (_, source))| source)
}

/// Return all version sources found at the specified path.
pub fn version_sources_from_path(path: &Path) -> Vec<Box<dyn VersionSource>> {
    let mut sources: Vec<Box<dyn VersionSource>> = Vec::new();

    for st in VersionSourceType::iter() {
        if let Some(source) = st.create_from_path(path) {
            sources.push(source);
        }
    }

    sources
}
