use std::borrow::Cow;

use serde_derive::Deserialize;

use crate::vut::VutError;

/// One or more regexes
#[derive(Clone, Debug, Deserialize)]
pub struct Regexes(pub Vec<String>);

impl Regexes {
    pub fn build_regexes(&self) -> Result<Vec<regex::Regex>, VutError> {
        let mut regexes: Vec<regex::Regex> = Vec::new();

        fn build_regex(pattern: &str) -> Result<regex::Regex, VutError> {
            let mut builder = regex::RegexBuilder::new(pattern);
            builder.multi_line(true);

            builder
                .build()
                .map_err(|err| VutError::Other(Cow::Owned(format!("Invalid regex '{}': {}", pattern, err.to_string()))))
        }

        for regex_str in self.0.iter() {
            regexes.push(build_regex(regex_str)?);
        }

        Ok(regexes)
    }
}
