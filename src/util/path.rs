use std::borrow::Cow;
use std::env;
use std::path::{Component, Path, PathBuf};

pub fn normalize_path(path: &Path) -> PathBuf {
    let path = if path.is_absolute() {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(env::current_dir().unwrap().join(path))
    };

    let mut new_path = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                new_path.pop();
            }
            c => {
                new_path.push(c);
            }
        };
    }

    new_path
}
