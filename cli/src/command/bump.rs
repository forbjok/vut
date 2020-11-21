use vut::project::{BumpVersion, Vut};
use vut::Version;

use super::{stderr_vut_callbacks, CommandError};

pub fn bump(bump_version: BumpVersion) -> Result<(), CommandError> {
    let mut vut = Vut::from_current_dir(Some(stderr_vut_callbacks()))?;

    let new_version: Version = vut.bump_version(bump_version)?;

    eprintln!("Version bumped to {}.", new_version.to_string());

    // Regenerate template output
    vut.generate_output()?;

    Ok(())
}
