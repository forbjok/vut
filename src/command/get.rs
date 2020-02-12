use std::io;

use serde_json;

use crate::vut::Vut;

use super::{CommandError, CommandErrorKind};

pub fn get(format: &str) -> Result<(), CommandError> {
    if let Some(vut) = Vut::from_current_dir()? {
        let version = vut.get_version();

        match format {
            "json" => get_json(&vut),
            _ => Err(CommandError::new(CommandErrorKind::Arguments, format!("Invalid format: {}!", format))),
        }
    } else {
        return Err(CommandError::new(CommandErrorKind::Config, "No VERSION file found!"));
    }
}

fn get_json(vut: &Vut) -> Result<(), CommandError> {
    let stdout = io::stdout();

    let template_input = vut.generate_template_input()?;

    serde_json::to_writer_pretty(stdout, &template_input.values)
        .map_err(|err| CommandError::new(CommandErrorKind::Other, format!("Error serializing values to JSON: {}", err.to_string())))?;

    Ok(())
}
