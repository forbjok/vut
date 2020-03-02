use std::borrow::Cow;
use std::path::Path;
use std::rc::Rc;

use regex::Regex;

use crate::util;
use crate::version::Version;
use crate::vut::VutError;

use super::FileUpdater;

pub struct CustomRegexFileUpdater {
    regexes: Rc<Vec<Regex>>,
}

impl CustomRegexFileUpdater {
    pub fn new(regexes: Vec<Regex>) -> Self {
        Self {
            regexes: Rc::new(regexes),
        }
    }
}

impl FileUpdater for CustomRegexFileUpdater {
    fn update_file(&self, file_path: &Path, encoding: Option<&str>, version: &Version) -> Result<(), VutError> {
        // Read text from file
        let mut text =
            util::read_text_file(file_path, encoding).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        // Get version string
        let version_str = version.to_string();

        // Iterate through all regexes, performing replacements for each one.
        for regex in self.regexes.iter() {
            text = regex
                .replace_all(&text, |caps: &regex::Captures| {
                    format!("{}{}{}", &caps[1], &version_str, &caps[3])
                })
                .into_owned();
        }

        // Write updated text to file
        util::write_text_file(file_path, text, encoding).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        Ok(())
    }
}
