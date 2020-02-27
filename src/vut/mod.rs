use std::borrow::Cow;
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use log::warn;
use strum_macros::EnumString;
use walkdir;

use crate::template::TemplateInput;
use crate::util;
use crate::version::Version;
use crate::version_source::{self, VersionSource};

pub mod config;
mod error;
mod generate_template;
mod ignore;
mod update_version_source;

pub use config::VutConfig;
pub use error::VutError;
use generate_template::*;
use ignore::*;
use update_version_source::*;

pub const VUT_CONFIG_FILENAME: &str = ".vutconfig.toml";

#[derive(Debug, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum BumpVersion {
    Major,
    Minor,
    Patch,
    Prerelease,
    Build,
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
        config::create_default_config_file(&config_file_path)?;

        let vut = if let Some(vut) = vut {
            // A version source was found, but no configuration file...
            // We need to support this in order to create a configuration file
            // for existing sources.

            Self {
                root_path: vut.root_path,
                config: VutConfig::from_str(config::VUT_CONFIG_DEFAULT)?,
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

            let source = if let Some(auth_vs_config) = &config.authoritative_version_source {
                // Authoritative version source configuration preset...

                // Path must be relative to the root path.
                if auth_vs_config.path.is_absolute() {
                    return Err(VutError::Config(Cow::Borrowed(
                        "Authoritative version source path must be relative!",
                    )));
                }

                // Construct absolute path.
                let auth_vs_path = root_path.join(&auth_vs_config.path);
                let auth_vs_path = util::normalize_path(&auth_vs_path);

                // If the specified path is outside the root path, return an error.
                if !auth_vs_path.starts_with(&root_path) {
                    return Err(VutError::Config(Cow::Borrowed(
                        "Authoritative version source path must be inside the root directory!",
                    )));
                }

                let auth_vs_type = &auth_vs_config._type;

                // Build HashSet containing only a single type name - the one specified in the configuration.
                // We need this to pass to the version source function below.
                let source_types: HashSet<String> = vec![auth_vs_type.clone()].into_iter().collect();

                // Try to get built-in version source.
                let mut version_sources =
                    version_source::specific_version_sources_from_path(&auth_vs_path, &source_types);

                // If no version source was found, try custom version sources.
                if version_sources.is_empty() {
                    let custom_source_types = build_custom_source_type_templates(&config)?;

                    if let Some(custom_source_type_template) = custom_source_types.get(auth_vs_type.as_str()) {
                        if let Some(source) = custom_source_type_template.instance_from_path(&auth_vs_path) {
                            version_sources.push(Box::new(source));
                        }
                    }
                }

                if version_sources.is_empty() {
                    // If still no version source was found, return an error.
                    return Err(VutError::NoVersionSource);
                } else if version_sources.len() > 1 {
                    // Since only one type is allowed to be specified,
                    // it should never be possible for more than one source to be returned.
                    return Err(VutError::Other(Cow::Borrowed("More than one authoritative version source was returned! This should never happen, and is probably caused by a bug.")));
                }

                // Return the first (and only) version source.
                version_sources.remove(0)
            } else {
                // No authoritative version source configuration specified, use root path.
                version_source::first_version_source_from_path(&root_path).ok_or_else(|| VutError::NoVersionSource)?
            };

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

        generate_template_input(&version)
    }

    pub fn generate_output(&self) -> Result<(), VutError> {
        let root_path = &self.root_path;

        let version = self.get_version()?;

        // Build ignore GlobSet from config
        let ignore_globset = build_ignore_globset(&self.config)?;

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

        if !self.config.version_source.is_empty() {
            update_version_sources(&self.config, root_path, &version, &dir_entries)?;
        }

        generate_template_output(&self.config, root_path, &version, &dir_entries)?;

        Ok(())
    }
}
