use vut::project::Vut;
use vut::Version;

use crate::error::*;

use super::*;

pub fn set(version: &str) -> Result<(), CliError> {
    let mut vut = Vut::from_current_dir(Some(stderr_vut_callbacks()))?;

    let new_version: Version = version.parse().map_err(|err| CliError::new(CliErrorKind::Other, err))?;

    vut.set_version(&new_version)?;

    eprintln!("Version set to {}.", new_version.to_string());

    // Regenerate template output
    vut.generate_output()?;

    Ok(())
}
