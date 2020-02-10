use std::path::{Path, PathBuf};

use crate::util;

pub fn locate_config_file(start_path: impl AsRef<Path>, filename: &str) -> Result<Option<PathBuf>, std::io::Error> {
    let start_path = util::normalize_path(start_path.as_ref());
    let mut current_path: Option<&Path> = Some(start_path.as_ref());

    while let Some(path) = current_path {
        let config_file = path.join(filename);

        if config_file.exists() {
            return Ok(Some(config_file));
        }

        current_path = path.parent();
    }

    Ok(None)
}
