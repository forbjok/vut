use std::path::Path;

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

pub fn locate_version_source_from(path: &Path) -> Option<Box<dyn VersionSource>> {
    if let Some(source) = VersionFileSource::locate_from_path(path) {
        Some(Box::new(source))
    } else if let Some(source) = CargoSource::locate_from_path(path) {
        Some(Box::new(source))
    } else if let Some(source) = NpmSource::locate_from_path(path) {
        Some(Box::new(source))
    } else {
        None
    }
}
