use std::env;

use crate::version::Version;
use crate::vut::Vut;

use super::{CommandError, CommandErrorKind};

pub fn init(version: Option<&str>) -> Result<(), CommandError> {
    let current_dir = env::current_dir()?;

    if let Some(vut) = Vut::from_path(&current_dir) {
        return Err(CommandError::new(CommandErrorKind::Other, format!("An existing version file was found at: {}", vut.get_version_file_path().to_string_lossy())));
    }

    let vut = Vut::new(current_dir);

    if vut.exists() {
        return Err(CommandError::new(CommandErrorKind::Other, "There is already a version file in this path."));
    }

    let version = match version {
        Some(v) => v.parse().unwrap(),
        None => Version::new(),
    };

    vut.set_version(&version)?;

    Ok(())
}
