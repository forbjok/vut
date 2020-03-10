use std::collections::BTreeMap;
use std::io;

use serde_json;

use vut::project::Vut;

use super::{stderr_vut_callbacks, CommandError, CommandErrorKind};

pub fn get(format: &str) -> Result<(), CommandError> {
    let vut = Vut::from_current_dir(Some(stderr_vut_callbacks()))?;

    match format {
        "json" => get_json(&vut),
        _ => Err(CommandError::new(
            CommandErrorKind::Arguments,
            format!("Invalid format: {}!", format),
        )),
    }
}

fn get_json(vut: &Vut) -> Result<(), CommandError> {
    let stdout = io::stdout();

    let template_input = vut.generate_template_input()?;

    // Copy values into a BTreeMap to sort them alphabetically
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in template_input.values.into_iter() {
        values.insert(k, v);
    }

    // Serialize pretty JSON to stdout
    serde_json::to_writer_pretty(stdout, &values).map_err(|err| {
        CommandError::new(
            CommandErrorKind::Other,
            format!("Error serializing values to JSON: {}", err.to_string()),
        )
    })?;

    Ok(())
}