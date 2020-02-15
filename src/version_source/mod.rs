use std::path::Path;

use crate::version::Version;
use crate::vut::VutError;

pub mod sources;

/// Trait representing the authoritative source of a project's version
pub trait VersionSource {
    fn get_root_path(&self) -> &Path;
    fn exists(&self) -> bool;
    fn get_version(&self) -> Result<Version, VutError>;
    fn set_version(&mut self, version: &Version) -> Result<(), VutError>;
}
