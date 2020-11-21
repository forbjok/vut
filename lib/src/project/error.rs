use std::borrow::Cow;
use std::io;
use std::path::PathBuf;

use crate::template::RenderTemplateError;
use crate::util;

pub enum VutError {
    AlreadyInit(PathBuf),
    OpenConfig(util::FileError),
    ReadConfig(io::Error),
    ParseConfig(Cow<'static, str>),
    Config(Cow<'static, str>),
    WriteConfig(io::Error),
    NoVersionSource,
    VersionNotFound,
    VersionFileOpen(util::FileError),
    VersionFileRead(io::Error),
    VersionFileWrite(io::Error),
    TemplateGenerate(RenderTemplateError),
    Other(Cow<'static, str>),
}
