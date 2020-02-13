use crate::version::Version;
use crate::vut::{BumpVersion, Vut};

use super::{CommandError, CommandErrorKind};

pub fn bump(bump_version: BumpVersion) -> Result<(), CommandError> {
    if let Some(vut) = Vut::from_current_dir()? {
        let new_version: Version = vut.bump_version(bump_version)?;

        println!("Version bumped to {}.", new_version.to_string());

        Ok(())
    } else {
        return Err(CommandError::new(CommandErrorKind::Config, "No version file found."));
    }
}
