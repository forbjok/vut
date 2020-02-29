use crate::version::Version;
use crate::vut::Vut;

use super::{stderr_vut_callbacks, CommandError, CommandErrorKind};

pub fn set(version: &str) -> Result<(), CommandError> {
    let mut vut = Vut::from_current_dir(Some(stderr_vut_callbacks()))?;

    let new_version: Version = version
        .parse()
        .map_err(|err| CommandError::new(CommandErrorKind::Other, err))?;

    vut.set_version(&new_version)?;

    eprintln!("Version set to {}.", new_version.to_string());

    // Regenerate template output
    vut.generate_output()?;

    Ok(())
}
