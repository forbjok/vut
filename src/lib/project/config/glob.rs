use std::borrow::Cow;

use serde_derive::Deserialize;

use crate::project::VutError;

/// One or more glob patterns.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum Globs {
    Single(String),
    Multiple(Vec<String>),
}

impl Globs {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Self::Single(value) => vec![value.clone()],
            Self::Multiple(values) => values.clone(),
        }
    }

    pub fn build_globset(&self) -> Result<globset::GlobSet, VutError> {
        let glob_strings = self.to_vec();

        let mut builder = globset::GlobSetBuilder::new();

        for glob_str in glob_strings.iter() {
            let glob = globset::Glob::new(glob_str).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;
            builder.add(glob);
        }

        let globset = builder
            .build()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        Ok(globset)
    }
}
