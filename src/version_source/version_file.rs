use std::borrow::Cow;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const VERSION_FILENAME: &'static str = "VERSION";

use crate::util;
use crate::version::Version;
use crate::version_source::VersionSource;
use crate::vut::VutError;

pub struct VersionFileSource {
    pub path: PathBuf,
    pub version_file_path: PathBuf,
}

impl VersionFileSource {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            version_file_path: path.to_path_buf(),
        }
    }

    pub fn locate_from_path(path: &Path) -> Option<Self> {
        util::locate_config_file(path, VERSION_FILENAME).map_or(None, |path| {
            let root_path = path.parent().unwrap();

            Some(Self {
                path: root_path.to_path_buf(),
                version_file_path: path.to_path_buf(),
            })
        })
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
            let mut file = util::open_file(&self.version_file_path)
                .map_err(|err| VutError::VersionFileOpen(err))?;

            let mut version_str = String::new();

            file.read_to_string(&mut version_str)
                .map_err(|err| VutError::VersionFileRead(err))?;

            version_str
        };

        let version = version_str.parse()
            .map_err(|err| VutError::Other(Cow::Owned(err)))?;

        Ok(version)
    }

    fn set_version(&mut self, version: &Version) -> Result<(), VutError> {
        let mut file = util::create_file(&self.version_file_path)
            .map_err(|err| VutError::VersionFileOpen(err))?;

        file.write(version.to_string().as_bytes())
            .map_err(|err| VutError::VersionFileWrite(err))?;

        Ok(())
    }
}
