use std::borrow::Cow;

use serde_derive::Deserialize;

use crate::vut::VutError;

/// One or more glob patterns.
#[derive(Clone, Debug, Deserialize)]
pub struct Globs(pub Vec<String>);

impl Globs {
    pub fn build_globset(&self) -> Result<globset::GlobSet, VutError> {
        let mut builder = globset::GlobSetBuilder::new();

        for glob in self.0.iter() {
            let glob = globset::Glob::new(glob).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;
            builder.add(glob);
        }

        let globset = builder
            .build()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        Ok(globset)
    }
}
