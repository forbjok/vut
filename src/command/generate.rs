use crate::vut::Vut;

use super::{CommandError, CommandErrorKind};

pub fn generate() -> Result<(), CommandError> {
    if let Some(vut) = Vut::from_current_dir() {
        println!("Generating templates...");
        vut.generate_output()?;

        Ok(())
    } else {
        return Err(CommandError::new(CommandErrorKind::Config, "No version file found."));
    }
}
