use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use strum_macros::EnumString;
use walkdir;

use crate::config;
use crate::template::{self, RenderTemplateError, TemplateInput};
use crate::util;
use crate::version::{self, Version};
use crate::version_source::{self, VersionSource};

const VUT_CONFIG_FILENAME: &str = ".vutconfig";

lazy_static! {
    static ref VUTEMPLATE_EXTENSION: &'static OsStr = OsStr::new("vutemplate");
}

#[derive(Debug, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum BumpVersion {
    Major,
    Minor,
    Patch,
    Prerelease,
    Build,
}

pub enum VutError {
    OpenConfig(util::FileError),
    ParseConfig(Cow<'static, str>),
    VersionFileOpen(util::FileError),
    VersionFileRead(io::Error),
    VersionFileWrite(io::Error),
    TemplateGenerate(RenderTemplateError),
    Other(Cow<'static, str>),
}

pub struct Vut {
    root_path: PathBuf,
    source: Box<dyn VersionSource>,
}

impl Vut {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let source = version_source::VersionFileSource::new(path);

        Self {
            root_path: path.to_path_buf(),
            source: Box::new(source),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Option<Self>, VutError> {
        let path = path.as_ref();

        let config_file_path = util::locate_config_file(path, VUT_CONFIG_FILENAME);

        let config = if let Some(path) = config_file_path {
            config::VutConfig::from_file(&path)?
        } else {
            config::VutConfig::default()
        };

        let source: Option<Box<dyn VersionSource>> = if let Some(source) = version_source::VersionFileSource::locate_from_path(path) {
            Some(Box::new(source))
        } else if let Some(source) = version_source::CargoSource::locate_from_path(path) {
            Some(Box::new(source))
        } else if let Some(source) = version_source::NpmSource::locate_from_path(path) {
            Some(Box::new(source))
        } else {
            None
        };

        Ok(if let Some(source) = source {
            let root_path = source.get_root_path();

            Some(Self {
                root_path: root_path.to_path_buf(),
                source,
            })
        } else {
            None
        })
    }

    pub fn from_current_dir() -> Result<Option<Self>, VutError> {
        let current_dir = env::current_dir().unwrap();

        Self::from_path(current_dir)
    }

    pub fn exists(&self) -> bool {
        self.source.exists()
    }

    pub fn get_root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn get_version(&self) -> Result<Version, VutError> {
        self.source.get_version()
    }

    pub fn set_version(&mut self, version: &Version) -> Result<(), VutError> {
        self.source.set_version(version)
    }

    pub fn bump_version(&mut self, bump_version: BumpVersion) -> Result<Version, VutError> {
        let version = self.get_version()?;

        let version = match bump_version {
            BumpVersion::Major => version.bump_major(),
            BumpVersion::Minor => version.bump_minor(),
            BumpVersion::Patch => version.bump_patch(),
            BumpVersion::Prerelease => version.bump_prerelease(),
            BumpVersion::Build => version.bump_build(),
        };

        self.source.set_version(&version)?;

        Ok(version)
    }

    pub fn generate_template_input(&self) -> Result<TemplateInput, VutError> {
        let version = self.get_version()?;

        let mut template_input = TemplateInput::new();

        let split_prerelease = version.prerelease.as_ref().map_or(None, |p| version::split_numbered_prerelease(p));
        let split_build = version.build.as_ref().map_or(None, |b| version::split_numbered_prerelease(b));

        template_input.values.insert("FullVersion".to_owned(), version.to_string());
        template_input.values.insert("Version".to_owned(), Version { build: None, ..version.clone() }.to_string());
        template_input.values.insert("MajorMinorPatch".to_owned(), format!("{}.{}.{}", version.major, version.minor, version.patch));
        template_input.values.insert("MajorMinor".to_owned(), format!("{}.{}", version.major, version.minor));
        template_input.values.insert("Major".to_owned(), format!("{}", version.major));
        template_input.values.insert("Minor".to_owned(), format!("{}", version.minor));
        template_input.values.insert("Patch".to_owned(), format!("{}", version.patch));
        template_input.values.insert("Prerelease".to_owned(), version.prerelease.as_ref().map_or("", |p| p).to_owned());
        template_input.values.insert("PrereleasePrefix".to_owned(), split_prerelease.and_then(|sp| Some(sp.0.to_owned())).unwrap_or_else(|| "".to_owned()));
        template_input.values.insert("PrereleaseNumber".to_owned(), split_prerelease.and_then(|sp| Some(format!("{}", sp.1))).unwrap_or_else(|| "".to_owned()));
        template_input.values.insert("Build".to_owned(), version.build.as_ref().map_or("", |b| b).to_owned());
        template_input.values.insert("BuildPrefix".to_owned(), split_build.and_then(|sp| Some(sp.0.to_owned())).unwrap_or_else(|| "".to_owned()));
        template_input.values.insert("BuildNumber".to_owned(), split_build.and_then(|sp| Some(format!("{}", sp.1))).unwrap_or_else(|| "".to_owned()));

        Ok(template_input)
    }

    pub fn locate_templates(&self) -> Vec<PathBuf> {
        let root_path = &self.root_path;

        let template_files: Vec<PathBuf> = walkdir::WalkDir::new(root_path).into_iter()
            // Filter known VCS metadata directories
            .filter_entry(|entry| !entry.file_name().to_str().map(|s| s == ".git" || s == ".hg").unwrap_or(false))
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.into_path())
            // Only include template files
            .filter(|path| {
                match path.extension() {
                    Some(ext) => ext == *VUTEMPLATE_EXTENSION,
                    None => false,
                }
            })
            // Make paths absolute
            .map(|path| util::normalize_path(&path).into_owned())
            // Exclude paths outside the root path
            .filter(|path| path.starts_with(&root_path))
            .collect();

        template_files
    }

    pub fn generate_output(&self) -> Result<(), VutError> {
        let template_input = self.generate_template_input()?;
        let template_files = self.locate_templates();

        let mut processed_files: Vec<PathBuf> = Vec::new();
        let mut generated_files: Vec<PathBuf> = Vec::new();

        for file in template_files {
            let generated_file = template::generate_template::<template::processor::VutProcessor>(&file, &template_input, None)
                .map_err(|err| VutError::TemplateGenerate(err))?;

            processed_files.push(file);
            generated_files.push(generated_file);
        }

        Ok(())
    }
}
