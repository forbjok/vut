
use crate::vut::{BumpVersion, Vut};

use super::{CommandError, CommandErrorKind};

pub fn bump(bump_version: BumpVersion) -> Result<(), CommandError> {
    if let Some(vut) = Vut::from_current_dir()? {
        let new_version = vut.bump_version(bump_version)?;

        println!("New version: {}", new_version.to_string());

        Ok(())
    } else {
        return Err(CommandError::new(CommandErrorKind::Config, "No VERSION file found!"));
    }
}
