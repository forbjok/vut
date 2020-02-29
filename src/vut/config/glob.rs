use std::borrow::Cow;

use serde_derive::Deserialize;

use crate::vut::VutError;

/// One or more glob patterns.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Globs {
    Single(String),
    Multiple(Vec<String>),
}

impl Globs {
    pub fn build_globset(&self) -> Result<globset::GlobSet, VutError> {
        let mut builder = globset::GlobSetBuilder::new();

        match self {
            Self::Single(glob) => {
                let glob = globset::Glob::new(glob).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;
                builder.add(glob);
            }
            Self::Multiple(globs) => {
                for glob in globs.iter() {
                    let glob = globset::Glob::new(glob).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;
                    builder.add(glob);
                }
            }
        };

        let globset = builder
            .build()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        Ok(globset)
    }
}
