use crate::vut::Vut;

use super::{stderr_vut_callbacks, CommandError};

pub fn generate() -> Result<(), CommandError> {
    let vut = Vut::from_current_dir(Some(stderr_vut_callbacks()))?;

    eprint!("Generating templates... ");

    vut.generate_output()?;

    eprintln!("Done.");

    Ok(())
}
