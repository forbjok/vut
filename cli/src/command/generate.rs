use vut::project::Vut;

use crate::error::*;
use crate::ui::StderrUiHandler;

pub fn generate() -> Result<(), CliError> {
    let mut ui = StderrUiHandler::new();

    let vut = Vut::from_current_dir(&mut ui)?;

    eprint!("Generating output... ");

    vut.generate_output(&mut ui)?;

    eprintln!("Done.");

    Ok(())
}
