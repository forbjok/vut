use std::borrow::Cow;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use globset;
use lazy_static::lazy_static;
use log::{debug, warn};
use strum_macros::EnumString;
use walkdir;

use crate::config::{self, UpdateSource, VutConfig};
use crate::template::{self, RenderTemplateError, TemplateInput};
use crate::util;
use crate::version::{self, Version};
use crate::version_source::{self, VersionSource};

const VUT_CONFIG_FILENAME: &str = ".vutconfig.toml";
const VUT_CONFIG_DEFAULT: &str = include_str!("default_config.toml");

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
    AlreadyInit(PathBuf),
    OpenConfig(util::FileError),
    ReadConfig(io::Error),
    ParseConfig(Cow<'static, str>),
    WriteConfig(io::Error),
    NoVersionSource,
    VersionFileOpen(util::FileError),
    VersionFileRead(io::Error),
    VersionFileWrite(io::Error),
    TemplateGenerate(RenderTemplateError),
    Other(Cow<'static, str>),
}

pub struct Vut {
    root_path: PathBuf,
    config: VutConfig,
    authoritative_version_source: Box<dyn VersionSource>,
}

impl Vut {
    pub fn init(path: impl AsRef<Path>, version: Option<&Version>) -> Result<Self, VutError> {
        let path = path.as_ref();

        // Check if there is an existing Vut configuration for this path
        let vut: Option<Vut> = match Self::from_path(path) {
            Ok(vut) => {
                let existing_root_path = vut.get_root_path();

                if existing_root_path != path || existing_root_path.join(VUT_CONFIG_FILENAME).exists() {
                    Err(VutError::AlreadyInit(vut.get_root_path().to_path_buf()))
                } else {
                    Ok(Some(vut))
                }
            }
            Err(VutError::NoVersionSource) => Ok(None),
            Err(err) => Err(err),
        }?;

        // Construct config file path
        let config_file_path = path.join(VUT_CONFIG_FILENAME);

        // Create configuration file with default content
        util::create_file(&config_file_path)
            .map_err(|err| VutError::OpenConfig(err))?
            .write(VUT_CONFIG_DEFAULT.trim().as_bytes())
            .map_err(|err| VutError::WriteConfig(err))?;

        let vut = if let Some(vut) = vut {
            // A version source was found, but no configuration file...
            // We need to support this in order to create a configuration file
            // for existing sources.

            Self {
                root_path: vut.root_path,
                config: VutConfig::from_str(VUT_CONFIG_DEFAULT)?,
                authoritative_version_source: vut.authoritative_version_source,
            }
        } else {
            // No version source was found...

            let version = version
                .map(|v| Cow::Borrowed(v))
                .unwrap_or_else(|| Cow::Owned(Version::new(0, 0, 0, None, None)));

            // Create a new version file source
            let mut source = version_source::VersionFileSource::new(path);

            // Set initial version
            source.set_version(&version)?;

            Self {
                root_path: path.to_path_buf(),
                config: VutConfig::default(),
                authoritative_version_source: Box::new(source),
            }
        };

        Ok(vut)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, VutError> {
        let path = path.as_ref();

        let config_file_path = util::locate_config_file(path, VUT_CONFIG_FILENAME);

        let config = if let Some(path) = config_file_path.as_ref() {
            config::VutConfig::from_file(path)?
        } else {
            config::VutConfig::default()
        };

        let (root_path, authoritative_version_source) = if let Some(config_file_path) = config_file_path.as_ref() {
            let root_path = config_file_path.parent().unwrap().to_path_buf();

            let source =
                version_source::first_version_source_from_path(&root_path).ok_or_else(|| VutError::NoVersionSource)?;

            (root_path, source)
        } else {
            // No config file found.
            // Fall back to trying to locate a version source instead.

            let source =
                version_source::locate_first_version_source_from(path).ok_or_else(|| VutError::NoVersionSource)?;

            // TODO: Find a better way to display deprecation warning.
            warn!("DEPRECATED: Authoritative version source present with no config file. Create a .vutconfig in the root of the project.");

            let root_path = source.get_path().to_path_buf();

            (root_path, source)
        };

        Ok(Self {
            root_path: root_path.to_path_buf(),
            config,
            authoritative_version_source,
        })
    }

    pub fn from_current_dir() -> Result<Self, VutError> {
        let current_dir = env::current_dir().unwrap();

        Self::from_path(current_dir)
    }

    pub fn exists(&self) -> bool {
        self.authoritative_version_source.exists()
    }

    pub fn get_root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn get_version(&self) -> Result<Version, VutError> {
        self.authoritative_version_source.get_version()
    }

    pub fn set_version(&mut self, version: &Version) -> Result<(), VutError> {
        self.authoritative_version_source.set_version(version)
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

        self.authoritative_version_source.set_version(&version)?;

        Ok(version)
    }

    pub fn generate_template_input(&self) -> Result<TemplateInput, VutError> {
        let version = self.get_version()?;

        let mut template_input = TemplateInput::new();

        let split_prerelease = version
            .prerelease
            .as_ref()
            .map_or(None, |p| version::split_numbered_prerelease(p));
        let split_build = version
            .build
            .as_ref()
            .map_or(None, |b| version::split_numbered_prerelease(b));

        template_input
            .values
            .insert("FullVersion".to_owned(), version.to_string());
        template_input.values.insert(
            "Version".to_owned(),
            Version {
                build: None,
                ..version.clone()
            }
            .to_string(),
        );
        template_input.values.insert(
            "MajorMinorPatch".to_owned(),
            format!("{}.{}.{}", version.major, version.minor, version.patch),
        );
        template_input
            .values
            .insert("MajorMinor".to_owned(), format!("{}.{}", version.major, version.minor));
        template_input
            .values
            .insert("Major".to_owned(), format!("{}", version.major));
        template_input
            .values
            .insert("Minor".to_owned(), format!("{}", version.minor));
        template_input
            .values
            .insert("Patch".to_owned(), format!("{}", version.patch));
        template_input.values.insert(
            "Prerelease".to_owned(),
            version.prerelease.as_ref().map_or("", |p| p).to_owned(),
        );
        template_input.values.insert(
            "PrereleasePrefix".to_owned(),
            split_prerelease
                .and_then(|sp| Some(sp.0.to_owned()))
                .unwrap_or_else(|| "".to_owned()),
        );
        template_input.values.insert(
            "PrereleaseNumber".to_owned(),
            split_prerelease
                .and_then(|sp| Some(format!("{}", sp.1)))
                .unwrap_or_else(|| "".to_owned()),
        );
        template_input
            .values
            .insert("Build".to_owned(), version.build.as_ref().map_or("", |b| b).to_owned());
        template_input.values.insert(
            "BuildPrefix".to_owned(),
            split_build
                .and_then(|sp| Some(sp.0.to_owned()))
                .unwrap_or_else(|| "".to_owned()),
        );
        template_input.values.insert(
            "BuildNumber".to_owned(),
            split_build
                .and_then(|sp| Some(format!("{}", sp.1)))
                .unwrap_or_else(|| "".to_owned()),
        );

        Ok(template_input)
    }

    fn build_template_globsets(&self) -> Result<Vec<(&config::Template, globset::GlobSet)>, VutError> {
        let mut template_globsets: Vec<(&config::Template, globset::GlobSet)> = Vec::new();

        for template in self.config.template.iter() {
            let patterns = match &template.pattern {
                config::Patterns::Single(v) => vec![v],
                config::Patterns::Multiple(v) => v.iter().collect(),
            };

            let mut globset = globset::GlobSetBuilder::new();

            for pattern in patterns {
                let glob = globset::Glob::new(&pattern).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;
                globset.add(glob);
            }

            let globset = globset
                .build()
                .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            template_globsets.push((&template, globset));
        }

        Ok(template_globsets)
    }

    /// Build a GlobSet from the ignore patterns in the configuration
    fn build_ignore_globset(&self) -> Result<globset::GlobSet, VutError> {
        let mut builder = globset::GlobSetBuilder::new();

        for pattern in self.config.ignore.iter() {
            let glob = globset::Glob::new(pattern).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            builder.add(glob);
        }

        let ignore_globset = builder
            .build()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        Ok(ignore_globset)
    }

    /// Build a GlobSet from the update_sources patterns in the configuration
    fn build_update_sources_globsets(&self) -> Result<Vec<(globset::GlobSet, Option<HashSet<String>>)>, VutError> {
        let mut update_version_sources: Vec<(globset::GlobSet, Option<HashSet<String>>)> = Vec::new();

        for update_source in self.config.update_sources.iter() {
            let (pattern, source_types) = match update_source {
                UpdateSource::Simple(path) => (path, None),
                UpdateSource::Detailed(us) => (&us.path, Some(&us.types)),
            };

            let glob = globset::Glob::new(&pattern).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            let globset = globset::GlobSetBuilder::new()
                .add(glob)
                .build()
                .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            update_version_sources.push((globset, source_types.map(|v| v.clone())));
        }

        Ok(update_version_sources)
    }

    /// Build a GlobSet from the exclude_sources patterns in the configuration
    fn build_exclude_sources_globset(&self) -> Result<globset::GlobSet, VutError> {
        let mut builder = globset::GlobSetBuilder::new();

        for pattern in self.config.exclude_sources.iter() {
            let glob = globset::Glob::new(pattern).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            builder.add(glob);
        }

        let exclude_sources_globset = builder
            .build()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        Ok(exclude_sources_globset)
    }

    fn generate_template_output(&self, dir_entries: &[walkdir::DirEntry]) -> Result<(), VutError> {
        let root_path = &self.root_path;

        let template_globsets = self.build_template_globsets()?;

        let template_input = self.generate_template_input()?;

        let mut processed_files: Vec<PathBuf> = Vec::new();
        let mut generated_files: Vec<PathBuf> = Vec::new();

        for (template, globset) in template_globsets {
            debug!("{:?}", template);

            let start_path: Cow<Path> = match &template.start_path {
                Some(p) => Cow::Owned(root_path.join(p)),
                None => Cow::Borrowed(root_path),
            };

            let output_path = match &template.output_path {
                Some(p) => Cow::Owned(root_path.join(p)),
                None => start_path.clone(),
            };

            let processor = template.processor.as_ref().map(|s| s.as_str()).unwrap_or("vut");
            let encoding = template.encoding.as_ref().map(|s| s.as_str());

            let template_files_iter = dir_entries
                .iter()
                .map(|entry| entry.path())
                // Exclude files outside the start path
                .filter(|path| path.starts_with(&start_path))
                // Only include template files
                .filter_map(|path| {
                    // Make path relative, as we only want to match on the path
                    // relative to the start path.
                    let rel_path = path.strip_prefix(&start_path).unwrap();

                    if globset.is_match(rel_path) {
                        Some((path, rel_path))
                    } else {
                        None
                    }
                });

            for (path, rel_path) in template_files_iter {
                debug!("Processing template {}", path.display());

                let output_file_name: &OsStr = path.file_stem().unwrap();
                let output_file_path = output_path.join(rel_path.with_file_name(output_file_name));

                template::generate_template_with_processor_name(
                    processor,
                    path,
                    &output_file_path,
                    &template_input,
                    encoding,
                )
                .map_err(|err| VutError::TemplateGenerate(err))?;

                processed_files.push(path.to_path_buf());
                generated_files.push(output_file_path);
            }
        }

        Ok(())
    }

    fn update_version_sources(&self, dir_entries: &[walkdir::DirEntry]) -> Result<(), VutError> {
        let root_path = &self.root_path;

        let update_sources_globsets = self.build_update_sources_globsets()?;
        let exclude_sources_globset = self.build_exclude_sources_globset()?;

        let mut sources: Vec<Box<dyn VersionSource>> = Vec::new();

        let dirs_iter = dir_entries
            .iter()
            .map(|entry| entry.path())
            // Only include directories
            .filter(|path| path.is_dir())
            // Exclude all paths matched in exclude sources
            .filter(|path| !exclude_sources_globset.is_match(path));

        for path in dirs_iter {
            // Make path relative, as we only want to match on the path
            // relative to the root.
            let rel_path = path.strip_prefix(root_path).unwrap();

            for (globset, source_types) in update_sources_globsets.iter() {
                if globset.is_match(&rel_path) {
                    // Check for version sources at this path
                    let mut new_sources = if let Some(source_types) = source_types {
                        version_source::specific_version_sources_from_path(&path, &source_types)
                    } else {
                        version_source::version_sources_from_path(&path)
                    };

                    // Append all found sources to the main list of sources
                    sources.append(&mut new_sources);
                }
            }
        }

        let version = self.get_version()?;

        for mut source in sources {
            source.set_version(&version)?;
        }

        Ok(())
    }

    pub fn generate_output(&self) -> Result<(), VutError> {
        let root_path = &self.root_path;

        // Build ignore GlobSet from config
        let ignore_globset = self.build_ignore_globset()?;

        let dir_entries: Vec<walkdir::DirEntry> = walkdir::WalkDir::new(root_path)
            .into_iter()
            // Filter known VCS metadata directories
            .filter_entry(|entry| {
                // Make path relative, as we only want to match on the path
                // relative to the root.
                let rel_path = entry.path().strip_prefix(root_path).unwrap();

                // Exclude paths matching any of the ignore glob patterns
                !ignore_globset.is_match(rel_path)
            })
            .filter_map(|entry| entry.ok())
            .collect();

        if !self.config.update_sources.is_empty() {
            self.update_version_sources(&dir_entries)?;
        }

        self.generate_template_output(&dir_entries)?;

        Ok(())
    }
}
