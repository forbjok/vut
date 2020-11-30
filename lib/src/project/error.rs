use std::borrow::Cow;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::template::RenderTemplateError;
use crate::util;

#[derive(Debug, Error)]
pub enum VutError {
    #[error("An existing configuration was found at: {0}")]
    AlreadyInit(PathBuf),
    #[error("Error opening config")]
    OpenConfig(util::FileError),
    #[error("Error reading config")]
    ReadConfig(io::Error),
    #[error("Error parsing config")]
    ParseConfig(Cow<'static, str>),
    #[error("Configuration error")]
    Config(Cow<'static, str>),
    #[error("Error writing config")]
    WriteConfig(io::Error),
    #[error("No version source found")]
    NoVersionSource,
    #[error("No version found in version source")]
    VersionNotFound,
    #[error("Error opening version source")]
    VersionFileOpen(util::FileError),
    #[error("Error reading version source")]
    VersionFileRead(io::Error),
    #[error("Error writing to version source")]
    VersionFileWrite(io::Error),
    #[error("Error generating template")]
    TemplateGenerate(RenderTemplateError),
    #[error("Error")]
    Other(Cow<'static, str>),
}
