use std::env;

use crate::version::Version;
use crate::vut::Vut;

use super::{CommandError, CommandErrorKind};

pub fn init(version: Option<&str>) -> Result<(), CommandError> {
    let current_dir = env::current_dir()?;

    let version: Option<Version> = match version {
        Some(s) => Some(
            s.parse()
                .map_err(|err| CommandError::new(CommandErrorKind::Other, err))?,
        ),
        None => None,
    };

    let vut = Vut::init(current_dir, version.as_ref())?;

    // Generate template output
    vut.generate_output()?;

    Ok(())
}
