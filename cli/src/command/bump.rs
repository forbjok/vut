use vut::project::{BumpVersion, Vut};
use vut::Version;

use crate::error::*;
use crate::ui::StderrUiHandler;

pub fn bump(bump_version: BumpVersion) -> Result<(), CliError> {
    let mut ui = StderrUiHandler::new();

    let mut vut = Vut::from_current_dir(&mut ui)?;

    let new_version: Version = vut.bump_version(bump_version, &mut ui)?;

    eprintln!("Version bumped to {}.", new_version);

    // Regenerate template output
    vut.generate_output(&mut ui)?;

    Ok(())
}
