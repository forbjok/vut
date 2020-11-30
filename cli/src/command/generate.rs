use vut::project::Vut;

use crate::error::*;

use super::*;

pub fn generate() -> Result<(), CliError> {
    let vut = Vut::from_current_dir(Some(stderr_vut_callbacks()))?;

    eprint!("Generating output... ");

    vut.generate_output()?;

    eprintln!("Done.");

    Ok(())
}
