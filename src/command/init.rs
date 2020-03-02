use std::env;

use crate::version::Version;
use crate::vut::{config, Vut};

use super::{stderr_vut_callbacks, CommandError, CommandErrorKind};

pub fn init(example: bool, version: Option<&str>) -> Result<(), CommandError> {
    let current_dir = env::current_dir()?;

    let version: Option<Version> = match version {
        Some(s) => Some(
            s.parse()
                .map_err(|err| CommandError::new(CommandErrorKind::Other, err))?,
        ),
        None => None,
    };

    let config_text = if example {
        config::VUT_CONFIG_EXAMPLE
    } else {
        config::VUT_CONFIG_DEFAULT
    };

    let vut = Vut::init(current_dir, version.as_ref(), Some(stderr_vut_callbacks()), config_text)?;

    eprintln!(
        "Initialized Vut project with version {} at {}.",
        vut.get_version()?.to_string(),
        vut.get_root_path().display()
    );

    // Generate template output
    vut.generate_output()?;

    Ok(())
}
