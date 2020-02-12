use crate::vut::Vut;

use super::{CommandError, CommandErrorKind};

pub fn set(version: &str) -> Result<(), CommandError> {
    if let Some(vut) = Vut::from_current_dir()? {
        vut.set_version(&version.parse().unwrap())?;

        Ok(())
    } else {
        return Err(CommandError::new(CommandErrorKind::Config, "No VERSION file found!"));
    }
}
