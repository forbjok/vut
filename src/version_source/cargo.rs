use std::borrow::Cow;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const CARGO_FILE_NAME: &'static str = "Cargo.toml";

use crate::util;
use crate::version::Version;
use crate::version_source::VersionSource;
use crate::vut::VutError;

use toml_edit;

pub struct CargoSource {
    pub path: PathBuf,
    pub cargo_file_path: PathBuf,
}

impl CargoSource {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            cargo_file_path: path.to_path_buf(),
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let cargo_file_path = path.join(CARGO_FILE_NAME);

        if cargo_file_path.exists() {
            Some(Self {
                path: path.to_path_buf(),
                cargo_file_path,
            })
        } else {
            None
        }
    }

    fn read_cargo_file(&self) -> Result<String, VutError> {
        let mut file = util::open_file(&self.cargo_file_path).map_err(|err| VutError::VersionFileOpen(err))?;

        let mut toml_str = String::new();

        file.read_to_string(&mut toml_str)
            .map_err(|err| VutError::VersionFileRead(err))?;

        Ok(toml_str)
    }

    fn write_cargo_file(&mut self, toml_str: &str) -> Result<(), VutError> {
        let mut file = util::create_file(&self.cargo_file_path).map_err(|err| VutError::VersionFileOpen(err))?;

        file.write(toml_str.as_bytes())
            .map_err(|err| VutError::VersionFileWrite(err))?;

        Ok(())
    }
}

impl VersionSource for CargoSource {
    fn get_path(&self) -> &Path {
        &self.path
    }

    fn exists(&self) -> bool {
        self.cargo_file_path.exists()
    }

    fn get_version(&self) -> Result<Version, VutError> {
        let version_str = {
            // Read TOML from cargo file
            let toml_str = self.read_cargo_file()?;

            // Parse as document
            let doc = toml_str
                .parse::<toml_edit::Document>()
                .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            // Get version string
            let version_str = doc["package"]["version"].as_str().unwrap();

            version_str.to_owned()
        };

        // Parse version string
        let version = version_str.parse().map_err(|err| VutError::Other(Cow::Owned(err)))?;

        Ok(version)
    }

    fn set_version(&mut self, version: &Version) -> Result<(), VutError> {
        // Read TOML from cargo file
        let toml_str = self.read_cargo_file()?;

        // Parse as document
        let mut doc = toml_str
            .parse::<toml_edit::Document>()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        // Replace version number
        doc["package"]["version"] = toml_edit::value(version.to_string());

        // Serialize updated document to string
        let toml_str = doc.to_string();

        // Overwrite cargo file
        self.write_cargo_file(&toml_str)?;

        Ok(())
    }
}
