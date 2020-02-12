use std::borrow::Cow;
use std::io;

use crate::util;

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
    Other,
}

impl CommandErrorKind {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Arguments => 1,
            Self::Config => 2,
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

/*
impl From<CollectValuesError> for CommandError {
    fn from(error: CollectValuesError) -> Self {
        CommandError {
            kind: CommandErrorKind::Other,
            description: error.description,
        }
    }
}

impl From<RenderTemplateError> for CommandError {
    fn from(error: RenderTemplateError) -> Self {
        CommandError {
            kind: CommandErrorKind::Other,
            description: error.description,
        }
    }
}
*/
