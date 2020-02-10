use std::env;
use std::io::Read;

use semver;
use strum_macros::EnumString;

use crate::util;
use super::{CommandError, CommandErrorKind};

#[derive(Debug, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum BumpStep {
    Major,
    Minor,
    Patch,
    Prerelease,
    Build,
}

pub fn bump(step: BumpStep) -> Result<(), CommandError> {
    let current_dir = env::current_dir().unwrap();
    let version_file = if let Some(version_file) = util::locate_config_file(current_dir, "VERSION").unwrap() {
        version_file
    } else {
        return Err(CommandError::new(CommandErrorKind::Config, "No VERSION file found!"));
    };

    let version_str = if let Ok(mut file) = util::open_file(version_file) {
        let mut version_str = String::new();
        file.read_to_string(&mut version_str)?;

        version_str
    } else {
        return Err(CommandError::new(CommandErrorKind::Config, "Cannot read VERSION file!"));
    };

    let mut version = semver::Version::parse(&version_str).unwrap();

    match step {
        BumpStep::Major => version.increment_major(),
        BumpStep::Minor => version.increment_minor(),
        BumpStep::Patch => version.increment_patch(),
        _ => { },
    };

    println!("New version: {}", version);

    Ok(())
}
