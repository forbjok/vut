use std::borrow::Cow;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const PROJECT_FILE_NAME: &str = "pyproject.toml";

use tracing::info;

use crate::project::VutError;
use crate::util;
use crate::version::Version;
use crate::version_source::VersionSource;

pub struct PyProjectSource {
    pub path: PathBuf,
    pub project_file_path: PathBuf,
}

impl PyProjectSource {
    pub fn from_path(path: &Path) -> Option<Self> {
        let project_file_path = path.join(PROJECT_FILE_NAME);

        if project_file_path.exists() {
            Some(Self {
                path: path.to_path_buf(),
                project_file_path,
            })
        } else {
            None
        }
    }

    fn read_project_file(&self) -> Result<String, VutError> {
        let mut file = util::open_file(&self.project_file_path).map_err(VutError::VersionFileOpen)?;

        let mut toml_str = String::new();

        file.read_to_string(&mut toml_str).map_err(VutError::VersionFileRead)?;

        Ok(toml_str)
    }

    fn write_project_file(&mut self, toml_str: &str) -> Result<(), VutError> {
        let mut file = util::create_file(&self.project_file_path).map_err(VutError::VersionFileOpen)?;

        file.write(toml_str.as_bytes()).map_err(VutError::VersionFileWrite)?;

        Ok(())
    }
}

impl VersionSource for PyProjectSource {
    fn get_path(&self) -> &Path {
        &self.path
    }

    fn exists(&self) -> bool {
        self.project_file_path.exists()
    }

    fn get_version(&self) -> Result<Version, VutError> {
        let version_str = {
            // Read TOML from project file
            let toml_str = self.read_project_file()?;

            // Parse as document
            let doc = toml_str
                .parse::<toml_edit::DocumentMut>()
                .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            // Get version string
            match doc["project"]["version"].as_str() {
                Some(version_str) => version_str.to_owned(),
                _ => {
                    info!("No version number found in '{}'.", self.project_file_path.display());
                    return Err(VutError::VersionNotFound);
                }
            }
        };

        // Parse version string
        let version = version_str.parse().map_err(|err| VutError::Other(Cow::Owned(err)))?;

        Ok(version)
    }

    fn set_version(&mut self, version: &Version) -> Result<(), VutError> {
        // Read TOML from project file
        let toml_str = self.read_project_file()?;

        // Parse as document
        let mut doc = toml_str
            .parse::<toml_edit::DocumentMut>()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        // If the file does not contain a [project] section, don't try to update version.
        if !doc.as_table().contains_key("project") {
            return Ok(());
        }

        // Replace version number
        doc["project"]["version"] = toml_edit::value(version.to_string());

        // Serialize updated document to string
        let toml_str = doc.to_string();

        // Overwrite cargo file
        self.write_project_file(&toml_str)?;

        Ok(())
    }
}
