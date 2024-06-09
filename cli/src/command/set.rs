use vut::project::Vut;
use vut::Version;

use crate::error::*;
use crate::ui::StderrUiHandler;

pub fn set(version: &str) -> Result<(), CliError> {
    let mut ui = StderrUiHandler::new();

    let mut vut = Vut::from_current_dir(&mut ui)?;

    let new_version: Version = version.parse().map_err(|err| CliError::new(CliErrorKind::Other, err))?;

    vut.set_version(&new_version, &mut ui)?;

    eprintln!("Version set to {}.", new_version);

    // Regenerate template output
    vut.generate_output(&mut ui)?;

    Ok(())
}
