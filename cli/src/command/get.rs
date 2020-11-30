use std::collections::BTreeMap;
use std::io;

use vut::project::Vut;
use vut::ui::*;

use crate::error::*;
use crate::ui::StderrUiHandler;

pub fn get(format: &str) -> Result<(), CliError> {
    let mut ui = StderrUiHandler::new();

    let vut = Vut::from_current_dir(&mut ui)?;

    match format {
        "json" => get_json(&vut, &mut ui),
        _ => Err(CliError::new(
            CliErrorKind::Arguments,
            format!("Invalid format: {}!", format),
        )),
    }
}

fn get_json(vut: &Vut, ui: &mut dyn VutUiHandler) -> Result<(), CliError> {
    let stdout = io::stdout();

    let template_input = vut.generate_template_input(ui)?;

    // Copy values into a BTreeMap to sort them alphabetically
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    for (k, v) in template_input.values.into_iter() {
        values.insert(k, v);
    }

    // Serialize pretty JSON to stdout
    serde_json::to_writer_pretty(stdout, &values).map_err(|err| {
        CliError::new(
            CliErrorKind::Other,
            format!("Error serializing values to JSON: {}", err.to_string()),
        )
    })?;

    Ok(())
}
