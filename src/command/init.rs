use std::env;

use crate::vut::Vut;

use super::{CommandError, CommandErrorKind};

pub fn init(version: &str) -> Result<(), CommandError> {
    let current_dir = env::current_dir()?;

    let vut = Vut::new(current_dir);

    if vut.exists() {
        return Err(CommandError::new(CommandErrorKind::Other, "There is already a VERSION file in this path."));
    }

    let version = version.parse().unwrap();

    vut.set_version(&version)?;

    Ok(())
}
