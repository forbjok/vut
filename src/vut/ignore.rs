use std::borrow::Cow;

use globset;

use super::{VutConfig, VutError};

/// Build a GlobSet from the ignore patterns in the configuration
pub fn build_ignore_globset(config: &VutConfig) -> Result<globset::GlobSet, VutError> {
    let mut builder = globset::GlobSetBuilder::new();

    for pattern in config.general.ignore.iter() {
        let glob = globset::Glob::new(pattern).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        builder.add(glob);
    }

    let ignore_globset = builder
        .build()
        .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

    Ok(ignore_globset)
}
