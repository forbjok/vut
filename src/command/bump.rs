use std::env;
use std::io::Read;

use strum_macros::EnumString;

use crate::util;
use crate::version::Version;
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

    let version: Version = version_str.parse().unwrap();

    let version = match step {
        BumpStep::Major => version.bump_major(),
        BumpStep::Minor => version.bump_minor(),
        BumpStep::Patch => version.bump_patch(),
        BumpStep::Prerelease => version.bump_prerelease(),
        BumpStep::Build => version.bump_build(),
    };

    println!("New version: {}", version.to_string());

    Ok(())
}
