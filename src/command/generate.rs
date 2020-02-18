use crate::vut::Vut;

use super::CommandError;

pub fn generate() -> Result<(), CommandError> {
    let vut = Vut::from_current_dir()?;

    eprint!("Generating templates... ");

    vut.generate_output()?;

    eprintln!("Done.");

    Ok(())
}
