use std::borrow::Cow;
use std::env;
use std::path::Path;

pub fn normalize_path(path: &Path) -> Cow<Path> {
    if path.is_absolute() {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(env::current_dir().unwrap().join(path))
    }
}
