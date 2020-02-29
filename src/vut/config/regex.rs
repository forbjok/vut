use std::borrow::Cow;

use serde_derive::Deserialize;

use crate::vut::VutError;

/// One or more regexes
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Regexes {
    Single(String),
    Multiple(Vec<String>),
}

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

        match self {
            Self::Single(pattern) => {
                regexes.push(build_regex(pattern)?);
            }
            Self::Multiple(patterns) => {
                for pattern in patterns.iter() {
                    regexes.push(build_regex(pattern)?);
                }
            }
        };

        Ok(regexes)
    }
}
