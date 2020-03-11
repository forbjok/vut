use std::borrow::Cow;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const VERSION_FILENAME: &str = "VERSION";

use crate::project::VutError;
use crate::util;
use crate::version::Version;
use crate::version_source::VersionSource;

pub struct VersionFileSource {
    pub path: PathBuf,
    pub version_file_path: PathBuf,
}

impl VersionFileSource {
    pub fn new(path: &Path) -> Self {
        let version_file_path = path.join(VERSION_FILENAME);

        Self {
            path: path.to_path_buf(),
            version_file_path,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let version_file_path = path.join(VERSION_FILENAME);

        if version_file_path.exists() {
            Some(Self {
                path: path.to_path_buf(),
                version_file_path,
            })
        } else {
            None
        }
    }
}

impl VersionSource for VersionFileSource {
    fn get_path(&self) -> &Path {
        &self.path
    }

    fn exists(&self) -> bool {
        self.version_file_path.exists()
    }

    fn get_version(&self) -> Result<Version, VutError> {
        let version_str = {
            let mut file = util::open_file(&self.version_file_path).map_err(VutError::VersionFileOpen)?;

            let mut version_str = String::new();

            file.read_to_string(&mut version_str)
                .map_err(VutError::VersionFileRead)?;

            version_str
        };

        let version = version_str.parse().map_err(|err| VutError::Other(Cow::Owned(err)))?;

        Ok(version)
    }

    fn set_version(&mut self, version: &Version) -> Result<(), VutError> {
        let mut file = util::create_file(&self.version_file_path).map_err(VutError::VersionFileOpen)?;

        file.write(version.to_string().as_bytes())
            .map_err(VutError::VersionFileWrite)?;

        Ok(())
    }
}
