use std::path::{Path, PathBuf};

use crate::util;

/// Traverse the filesystem outwards from the specified start path,
/// checking if the specified filename exists in each directory.
/// Returns the path of the first file found, or None is no file is found.
pub fn locate_config_file(start_path: impl AsRef<Path>, filename: &str) -> Option<PathBuf> {
    find_outwards(start_path, move |path| {
        let config_file = path.join(filename);

        if config_file.exists() { Some(config_file) } else { None }
    })
    .map(|(_, rv)| rv)
}

/// Traverse the filesystem outwards from the specified start path,
/// executing the predicate function in each directory.
/// Returns the path and value of the first predicate that returns Some.
pub fn find_outwards<T, P>(start_path: impl AsRef<Path>, predicate: P) -> Option<(PathBuf, T)>
where
    P: Fn(&Path) -> Option<T>,
{
    let start_path = util::normalize_path(start_path.as_ref());
    let mut current_path: Option<&Path> = Some(start_path.as_ref());

    while let Some(path) = current_path {
        if let Some(rv) = predicate(path) {
            return Some((path.to_path_buf(), rv));
        }

        current_path = path.parent();
    }

    None
}
