use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use strum_macros::EnumString;

use crate::template::TemplateInput;
use crate::util;
use crate::version::Version;
use crate::version_source::{self, VersionSource, VersionSourceType};

pub mod config;
mod error;
mod generate_template;
mod update_file;
mod update_version_source;

use crate::ui::{UiEvent, VutUiHandler};

pub use config::VutConfig;
pub use error::VutError;
use generate_template::*;
use update_file::*;
use update_version_source::*;

pub const VUT_CONFIG_FILENAME: &str = "vut.toml";

#[derive(Clone, Debug, EnumString)]
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
    pub fn init(
        path: impl AsRef<Path>,
        version: Option<&Version>,
        config_text: &str,
        force: bool,
        _ui: &mut dyn VutUiHandler,
    ) -> Result<Self, VutError> {
        let path = path.as_ref();

        // Check if there is an existing Vut configuration for this path.
        if let Some(existing_config_file_path) = util::locate_config_file(path, VUT_CONFIG_FILENAME) {
            // An existing configuration file was found...

            let existing_root_path = existing_config_file_path.parent().unwrap();

            if force && existing_root_path != path {
                // Force is enabled, and the config found is not in the same directory we are
                // trying to initialize.
                // This is allowed.
            } else {
                // Otherwise, disallow and fail.
                return Err(VutError::AlreadyInit(existing_root_path.to_path_buf()));
            }
        }

        // Construct config file path
        let config_file_path = path.join(VUT_CONFIG_FILENAME);

        let (avs_type, authoritative_version_source) = match version_source::first_version_source_from_path(path) {
            Some((source_type, source)) => {
                // A version source was found at the current directory. Use it.
                (source_type, source)
            }
            _ => {
                // No version source was found...

                let version = version
                    .map(Cow::Borrowed)
                    .unwrap_or_else(|| Cow::Owned(Version::new(0, 0, 0, None, None)));

                // Create a new version file source
                let mut source = version_source::VersionFileSource::new(path);

                // Set initial version
                source.set_version(&version)?;

                (VersionSourceType::Vut, Box::new(source) as Box<dyn VersionSource>)
            }
        };

        // Customize and create initial config
        let config = {
            let mut doc = config_text
                .parse::<toml_edit::DocumentMut>()
                .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            // Set authoritative version source type
            doc["authoritative-version-source"]["type"] = toml_edit::value(avs_type.as_ref());

            // Serialize updated document to string
            let toml_str = doc.to_string();

            // Create configuration file
            config::create_config_file(&config_file_path, &toml_str)?
        };

        Ok(Self {
            root_path: path.to_path_buf(),
            config,
            authoritative_version_source,
        })
    }

    pub fn from_path(path: impl AsRef<Path>, ui: &mut dyn VutUiHandler) -> Result<Self, VutError> {
        let path = path.as_ref();

        let config_file_path = util::locate_config_file(path, VUT_CONFIG_FILENAME);

        let config = if let Some(path) = config_file_path.as_ref() {
            config::VutConfig::from_file(path)?
        } else {
            config::VutConfig::legacy()
        };

        let (root_path, authoritative_version_source) = if let Some(config_file_path) = config_file_path.as_ref() {
            let root_path = config_file_path.parent().unwrap().to_path_buf();

            let auth_vs_type = config
                .authoritative_version_source
                ._type
                .as_deref()
                .unwrap_or_else(|| VersionSourceType::Vut.as_ref());

            let auth_vs_path: Cow<Path> = match &config.authoritative_version_source.path {
                Some(auth_vs_path) => {
                    // Authoritative version source configuration preset...

                    // Path must be relative to the root path.
                    if auth_vs_path.is_absolute() {
                        return Err(VutError::Config(Cow::Borrowed(
                            "Authoritative version source path must be relative!",
                        )));
                    }

                    // Construct absolute path.
                    let auth_vs_path = root_path.join(auth_vs_path);
                    let auth_vs_path = util::normalize_path(auth_vs_path);

                    // If the specified path is outside the root path, return an error.
                    if !auth_vs_path.starts_with(&root_path) {
                        return Err(VutError::Config(Cow::Borrowed(
                            "Authoritative version source path must be inside the root directory!",
                        )));
                    }

                    Cow::Owned(auth_vs_path)
                }
                _ => {
                    // No authoritative version source configuration specified, use root path.
                    Cow::Borrowed(&root_path)
                }
            };

            let source = {
                // Try to get built-in version source.
                let mut version_sources = Vec::new();

                if let Ok(vst) = VersionSourceType::from_str(auth_vs_type) {
                    if let Some(source) = vst.create_from_path(&auth_vs_path) {
                        version_sources.push(source);
                    }
                } else {
                    let custom_source_types = CustomSourceTypes::from_config(&config)?;

                    if let Some(source) = custom_source_types
                        .version_source_from_path(&auth_vs_path, auth_vs_type)
                        .into_iter()
                        .next()
                    {
                        version_sources.push(source);
                    }
                }

                if version_sources.is_empty() {
                    // If still no version source was found, return an error.
                    return Err(VutError::NoVersionSource);
                } else if version_sources.len() > 1 {
                    // Since only one type is allowed to be specified,
                    // it should never be possible for more than one source to be returned.
                    return Err(VutError::Other(Cow::Borrowed(
                        "More than one authoritative version source was returned! This should never happen, and is probably caused by a bug.",
                    )));
                }

                // Return the first (and only) version source.
                version_sources.remove(0)
            };

            (root_path, source)
        } else {
            // No config file found.
            // Fall back to trying to locate a version source instead.

            let source = version_source::locate_first_version_source_from(path).ok_or(VutError::NoVersionSource)?;

            // Display deprecation warning.
            ui.event(&UiEvent::DeprecationWarning("Authoritative version source present with no config file. Use 'vut init' to create a configuration file in the project root.".into()));

            let root_path = source.get_path().to_path_buf();

            (root_path, source)
        };

        Ok(Self {
            root_path,
            config,
            authoritative_version_source,
        })
    }

    pub fn from_current_dir(ui: &mut dyn VutUiHandler) -> Result<Self, VutError> {
        let current_dir = env::current_dir().unwrap();

        Self::from_path(current_dir, ui)
    }

    pub fn exists(&self) -> bool {
        self.authoritative_version_source.exists()
    }

    pub fn get_root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn get_version(&self, _ui: &mut dyn VutUiHandler) -> Result<Version, VutError> {
        self.authoritative_version_source.get_version()
    }

    pub fn set_version(&mut self, version: &Version, _ui: &mut dyn VutUiHandler) -> Result<(), VutError> {
        self.authoritative_version_source.set_version(version)
    }

    pub fn bump_version(&mut self, bump_version: BumpVersion, ui: &mut dyn VutUiHandler) -> Result<Version, VutError> {
        let version = self.get_version(ui)?;

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

    pub fn generate_template_input(&self, ui: &mut dyn VutUiHandler) -> Result<TemplateInput, VutError> {
        let version = self.get_version(ui)?;

        generate_template_input(&version)
    }

    pub fn generate_output(&self, ui: &mut dyn VutUiHandler) -> Result<(), VutError> {
        let root_path = &self.root_path;

        let version = self.get_version(ui)?;

        // Build ignore GlobSet from config
        let ignore_globset = match &self.config.general.ignore {
            Some(ignore) => ignore.build_globset()?,
            _ => globset::GlobSet::empty(),
        };

        let dir_entries: Vec<walkdir::DirEntry> = walkdir::WalkDir::new(root_path)
            .into_iter()
            // Filter ignored paths and paths containing
            // other Vut configurations.
            .filter_entry(|entry| {
                if entry.file_type().is_dir() {
                    let path = entry.path();

                    // Exclude directories containing a Vut configuration file,
                    // unless it is our own root directory.
                    if path.join(VUT_CONFIG_FILENAME).is_file() && path != root_path {
                        return false;
                    }

                    // Make path relative, as we only want to match on the path
                    // relative to the root.
                    let rel_path = path.strip_prefix(root_path).unwrap();

                    // Exclude paths matching any of the ignore glob patterns
                    !ignore_globset.is_match(rel_path)
                } else {
                    true
                }
            })
            .filter_map(|entry| entry.ok())
            .collect();

        // Get template input
        let template_input = generate_template_input(&version)?;

        // Update version sources.
        if !self.config.update_version_sources.is_empty() {
            update_version_sources(&self.config, root_path, &version, &dir_entries)?;
        }

        // Update files.
        update_files(&self.config, root_path, &dir_entries, &template_input)?;

        // Generate template output.
        generate_template_output(&self.config, root_path, &dir_entries, &template_input)?;

        Ok(())
    }
}
