use crate::version::Version;
use crate::vut::Vut;

use super::{CommandError, CommandErrorKind};

pub fn set(version: &str) -> Result<(), CommandError> {
    if let Some(vut) = Vut::from_current_dir() {
        let new_version: Version = version.parse()
            .map_err(|err| CommandError::new(CommandErrorKind::Other, err))?;

        vut.set_version(&new_version)?;

        println!("Version set to {}.", new_version.to_string());

        // Regenerate template output
        vut.generate_output()?;

        Ok(())
    } else {
        return Err(CommandError::new(CommandErrorKind::Config, "No version file found."));
    }
}
