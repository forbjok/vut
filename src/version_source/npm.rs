use std::borrow::Cow;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const PACKAGE_FILE_NAME: &'static str = "package.json";

use crate::util;
use crate::version::Version;
use crate::version_source::VersionSource;
use crate::vut::VutError;

use serde_json;

pub struct NpmSource {
    pub root_path: PathBuf,
    pub package_file_path: PathBuf,
}

impl NpmSource {
    pub fn new(path: &Path) -> Self {
        Self {
            root_path: path.to_path_buf(),
            package_file_path: path.to_path_buf(),
        }
    }

    pub fn locate_from_path(path: &Path) -> Option<Self> {
        util::locate_config_file(path, PACKAGE_FILE_NAME).map_or(None, |path| {
            let root_path = path.parent().unwrap();

            Some(Self {
                root_path: root_path.to_path_buf(),
                package_file_path: path.to_path_buf(),
            })
        })
    }

    fn read_package_file(&self) -> Result<String, VutError> {
        let mut file = util::open_file(&self.package_file_path)
            .map_err(|err| VutError::VersionFileOpen(err))?;

        let mut json_str = String::new();

        file.read_to_string(&mut json_str)
            .map_err(|err| VutError::VersionFileRead(err))?;

        Ok(json_str)
    }

    fn write_package_file(&mut self, json_str: &str) -> Result<(), VutError> {
        let mut file = util::create_file(&self.package_file_path)
            .map_err(|err| VutError::VersionFileOpen(err))?;

        file.write(json_str.as_bytes())
            .map_err(|err| VutError::VersionFileWrite(err))?;

        Ok(())
    }
}

impl VersionSource for NpmSource {
    fn get_root_path(&self) -> &Path {
        &self.root_path
    }

    fn exists(&self) -> bool {
        self.package_file_path.exists()
    }

    fn get_version(&self) -> Result<Version, VutError> {
        let version_str = {
            // Read package file to JSON string
            let json_str = self.read_package_file()?;

            // Deserialize into JSON Value
            let package: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            // Get version string
            let version_str = package["version"].as_str()
                .ok_or_else(|| VutError::Other(Cow::Borrowed("No version property found!")))?;

            version_str.to_owned()
        };

        // Parse version string
        let version = version_str.parse()
            .map_err(|err| VutError::Other(Cow::Owned(err)))?;

        Ok(version)
    }

    fn set_version(&mut self, version: &Version) -> Result<(), VutError> {
        // Read package file to JSON string
        let json_str = self.read_package_file()?;

        // Deserialize into JSON Value
        let mut package: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        // Replace version number
        package["version"] = serde_json::Value::from(version.to_string());

        // Serialize updated document to string
        let json_str = serde_json::to_string_pretty(&package)
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        // Overwrite package file
        self.write_package_file(&json_str)?;

        Ok(())
    }
}