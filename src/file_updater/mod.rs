use std::path::Path;

use crate::version::Version;
use crate::vut::VutError;

mod custom_regex;

pub use custom_regex::*;

pub trait FileUpdater {
    fn update_file(&self, path: &Path, version: &Version) -> Result<(), VutError>;
}
