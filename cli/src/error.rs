use std::borrow::Cow;
use std::io;

use vut::project::VutError;
use vut::util;

#[derive(Debug)]
pub enum CliErrorKind {
    Arguments,
    Config,
    NoVersionSource,
    Other,
}

impl CliErrorKind {
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
pub struct CliError {
    pub kind: CliErrorKind,
    pub description: Cow<'static, str>,
}

impl CliError {
    pub fn new<S: Into<Cow<'static, str>>>(kind: CliErrorKind, description: S) -> CliError {
        CliError {
            kind,
            description: description.into(),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        CliError {
            kind: CliErrorKind::Other,
            description: Cow::Owned(error.to_string()),
        }
    }
}

impl From<util::FileError> for CliError {
    fn from(error: util::FileError) -> Self {
        CliError {
            kind: CliErrorKind::Other,
            description: Cow::Owned(format!("File not found: {}", error.path.display())),
        }
    }
}

impl From<VutError> for CliError {
    fn from(error: VutError) -> Self {
        let kind = match error {
            VutError::AlreadyInit(_) => CliErrorKind::Other,
            VutError::OpenConfig(_) => CliErrorKind::Config,
            VutError::ReadConfig(_) => CliErrorKind::Config,
            VutError::ParseConfig(_) => CliErrorKind::Config,
            VutError::Config(_) => CliErrorKind::Config,
            VutError::WriteConfig(_) => CliErrorKind::Config,
            VutError::NoVersionSource => CliErrorKind::NoVersionSource,
            VutError::VersionNotFound => CliErrorKind::NoVersionSource,
            VutError::VersionFileOpen(_) => CliErrorKind::Other,
            VutError::VersionFileRead(_) => CliErrorKind::Other,
            VutError::VersionFileWrite(_) => CliErrorKind::Other,
            VutError::TemplateGenerate(_) => CliErrorKind::Other,
            VutError::Other(_) => CliErrorKind::Other,
        };

        Self {
            kind,
            description: error.to_string().into(),
        }
    }
}
