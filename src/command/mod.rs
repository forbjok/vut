use std::borrow::Cow;
use std::io;

use crate::util;
use crate::vut::VutError;

mod bump;
mod generate;
mod get;
mod init;
mod set;

pub use bump::*;
pub use generate::*;
pub use get::*;
pub use init::*;
pub use set::*;

#[derive(Debug)]
pub enum CommandErrorKind {
    Arguments,
    Config,
    NoVersionSource,
    Other,
}

impl CommandErrorKind {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Arguments => 1,
            Self::Config => 2,
            Self::NoVersionSource => 3,
            Self::Other => 101,
        }
    }
}

#[derive(Debug)]
pub struct CommandError {
    pub kind: CommandErrorKind,
    pub description: Cow<'static, str>,
}

impl CommandError {
    pub fn new<S: Into<Cow<'static, str>>>(kind: CommandErrorKind, description: S) -> CommandError {
        CommandError {
            kind,
            description: description.into(),
        }
    }
}

impl From<io::Error> for CommandError {
    fn from(error: io::Error) -> Self {
        CommandError {
            kind: CommandErrorKind::Other,
            description: Cow::Owned(error.to_string()),
        }
    }
}

impl From<util::FileError> for CommandError {
    fn from(error: util::FileError) -> Self {
        CommandError {
            kind: CommandErrorKind::Other,
            description: Cow::Owned(format!("File not found: {}", error.path.to_string_lossy())),
        }
    }
}

impl From<VutError> for CommandError {
    fn from(error: VutError) -> Self {
        match error {
            VutError::AlreadyInit(root_path) => CommandError::new(
                CommandErrorKind::Other,
                format!(
                    "An existing configuration was found at: {}",
                    root_path.to_string_lossy()
                ),
            ),
            VutError::OpenConfig(err) => CommandError::new(
                CommandErrorKind::Config,
                format!("Error opening config file: {}", err.to_string()),
            ),
            VutError::ReadConfig(err) => {
                CommandError::new(CommandErrorKind::Config, format!("Error reading config file: {}", err))
            }
            VutError::ParseConfig(err) => CommandError::new(
                CommandErrorKind::Config,
                format!("Error parsing configuration: {}", err),
            ),
            VutError::NoVersionSource => {
                CommandError::new(CommandErrorKind::NoVersionSource, "No version source found.")
            }
            VutError::VersionFileOpen(err) => CommandError::new(
                CommandErrorKind::Other,
                format!("Error opening version file: {}", err.to_string()),
            ),
            VutError::VersionFileRead(err) => CommandError::new(
                CommandErrorKind::Other,
                format!("Error reading version file: {}", err.to_string()),
            ),
            VutError::VersionFileWrite(err) => CommandError::new(
                CommandErrorKind::Other,
                format!("Error writing version file: {}", err.to_string()),
            ),
            VutError::TemplateGenerate(err) => CommandError::new(
                CommandErrorKind::Other,
                format!("Error generating templates: {}", err.to_string()),
            ),
            VutError::Other(err) => CommandError::new(CommandErrorKind::Other, err.to_string()),
        }
    }
}
