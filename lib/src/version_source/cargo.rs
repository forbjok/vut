use std::borrow::Cow;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const CARGO_FILE_NAME: &str = "Cargo.toml";

use tracing::info;

use crate::project::VutError;
use crate::util;
use crate::version::Version;
use crate::version_source::VersionSource;

pub struct CargoSource {
    pub path: PathBuf,
    pub cargo_file_path: PathBuf,
}

impl CargoSource {
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
        let mut file = util::open_file(&self.cargo_file_path).map_err(VutError::VersionFileOpen)?;

        let mut toml_str = String::new();

        file.read_to_string(&mut toml_str).map_err(VutError::VersionFileRead)?;

        Ok(toml_str)
    }

    fn write_cargo_file(&mut self, toml_str: &str) -> Result<(), VutError> {
        let mut file = util::create_file(&self.cargo_file_path).map_err(VutError::VersionFileOpen)?;

        file.write(toml_str.as_bytes()).map_err(VutError::VersionFileWrite)?;

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
                .parse::<toml_edit::DocumentMut>()
                .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            // Get version string
            match doc["package"]["version"].as_str() {
                Some(version_str) => version_str.to_owned(),
                _ => {
                    info!("No version number found in '{}'. This Cargo.toml may be a workspace, and cannot be used as a version source.", self.cargo_file_path.display());
                    return Err(VutError::VersionNotFound);
                }
            }
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
            .parse::<toml_edit::DocumentMut>()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        // If the file does not contain a [package] section, don't try to update version.
        // This will typically be the case if the Cargo.toml is a workspace.
        if !doc.as_table().contains_key("package") {
            return Ok(());
        }

        // Replace version number
        doc["package"]["version"] = toml_edit::value(version.to_string());

        // Serialize updated document to string
        let toml_str = doc.to_string();

        // Overwrite cargo file
        self.write_cargo_file(&toml_str)?;

        Ok(())
    }
}
