use std::path::Path;

use crate::util;
use crate::version::Version;
use crate::vut::VutError;

mod cargo;
mod npm;
mod version_file;

pub use cargo::*;
pub use npm::*;
pub use version_file::*;

/// Trait representing the authoritative source of a project's version
pub trait VersionSource {
    fn get_path(&self) -> &Path;
    fn exists(&self) -> bool;
    fn get_version(&self) -> Result<Version, VutError>;
    fn set_version(&mut self, version: &Version) -> Result<(), VutError>;
}

pub fn locate_version_source_from(start_path: &Path) -> Option<Box<dyn VersionSource>> {
    util::find_outwards(start_path, |path| {
        let source: Option<Box<dyn VersionSource>> = if let Some(source) = VersionFileSource::from_path(path) {
            Some(Box::new(source))
        } else if let Some(source) = CargoSource::from_path(path) {
            Some(Box::new(source))
        } else if let Some(source) = NpmSource::from_path(path) {
            Some(Box::new(source))
        } else {
            None
        };

        source
    }).map(|(_, source)| source)
}
