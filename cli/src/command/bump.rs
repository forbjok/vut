use vut::project::{BumpVersion, Vut};
use vut::Version;

use crate::error::*;

use super::*;

pub fn bump(bump_version: BumpVersion) -> Result<(), CliError> {
    let mut vut = Vut::from_current_dir(Some(stderr_vut_callbacks()))?;

    let new_version: Version = vut.bump_version(bump_version)?;

    eprintln!("Version bumped to {}.", new_version.to_string());

    // Regenerate template output
    vut.generate_output()?;

    Ok(())
}
