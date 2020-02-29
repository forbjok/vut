use std::borrow::Cow;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use regex::Regex;

use crate::util;
use crate::version::Version;
use crate::version_source::VersionSource;
use crate::vut::VutError;

pub struct CustomRegexSourceTemplate {
    file_name: String,
    regex: Rc<Regex>,
}

pub struct CustomRegexSource {
    path: PathBuf,
    file_path: PathBuf,
    regex: Rc<Regex>,
}

impl CustomRegexSourceTemplate {
    pub fn new(file_name: &str, regex: Regex) -> Self {
        Self {
            file_name: file_name.to_owned(),
            regex: Rc::new(regex),
        }
    }

    pub fn instance_from_path(&self, path: &Path) -> Option<CustomRegexSource> {
        let file_path = path.join(&self.file_name);

        if file_path.exists() {
            Some(CustomRegexSource {
                path: path.to_path_buf(),
                file_path,
                regex: self.regex.clone(),
            })
        } else {
            None
        }
    }
}

impl<'a> CustomRegexSource {
    fn read_file(&self) -> Result<String, VutError> {
        let mut file = util::open_file(&self.file_path).map_err(|err| VutError::VersionFileOpen(err))?;

        let mut text = String::new();

        file.read_to_string(&mut text)
            .map_err(|err| VutError::VersionFileRead(err))?;

        Ok(text)
    }

    fn write_file(&mut self, text: &str) -> Result<(), VutError> {
        let mut file = util::create_file(&self.file_path).map_err(|err| VutError::VersionFileOpen(err))?;

        file.write(text.as_bytes())
            .map_err(|err| VutError::VersionFileWrite(err))?;

        Ok(())
    }
}

impl<'a> VersionSource for CustomRegexSource {
    fn get_path(&self) -> &Path {
        &self.path
    }

    fn exists(&self) -> bool {
        self.file_path.exists()
    }

    fn get_version(&self) -> Result<Version, VutError> {
        let version_str = {
            // Read text from file
            let text = self.read_file()?;

            // Get version string using regex
            let version_str = if let Some(caps) = self.regex.captures(&text) {
                caps[2].to_owned()
            } else {
                return Err(VutError::Other(Cow::Borrowed("Error parsing file using custom regex!")));
            };

            version_str
        };

        // Parse version string
        let version = version_str.parse().map_err(|err| VutError::Other(Cow::Owned(err)))?;

        Ok(version)
    }

    fn set_version(&mut self, version: &Version) -> Result<(), VutError> {
        // Read text from file
        let text = self.read_file()?;

        let version_str = version.to_string();

        // Replace version number
        let text = self.regex.replace(&text, |caps: &regex::Captures| {
            format!("{}{}{}", &caps[1], &version_str, &caps[3])
        });

        // Overwrite cargo file
        self.write_file(&text)?;

        Ok(())
    }
}
