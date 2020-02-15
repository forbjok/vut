use std::env;

use crate::version::Version;
use crate::vut::Vut;

use super::{CommandError, CommandErrorKind};

pub fn init(version: Option<&str>) -> Result<(), CommandError> {
    let current_dir = env::current_dir()?;

    if let Some(vut) = Vut::from_path(&current_dir) {
        return Err(CommandError::new(CommandErrorKind::Other, format!("An existing version file was found at: {}", vut.get_root_path().to_string_lossy())));
    }

    let mut vut = Vut::new(current_dir);

    let version = match version {
        Some(v) => v.parse().map_err(|err| CommandError::new(CommandErrorKind::Other, err))?,
        None => Version::new(0, 0, 0, None, None),
    };

    vut.set_version(&version)?;

    // Generate template output
    vut.generate_output()?;

    Ok(())
}
