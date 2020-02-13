use std::borrow::Cow;
use std::env;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use glob;
use strum_macros::EnumString;

use crate::template::{self, RenderTemplateError, TemplateInput};
use crate::util;
use crate::version::{self, Version};

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
    VersionFileOpen(util::FileError),
    VersionFileRead(io::Error),
    VersionFileWrite(io::Error),
    TemplateGenerate(RenderTemplateError),
    Other(Cow<'static, str>),
}

pub struct Vut {
    root_path: PathBuf,
    version_file_path: PathBuf,
}

impl Vut {
    const VERSION_FILENAME: &'static str = "VERSION";

    pub fn new(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        Self {
            root_path: path.to_path_buf(),
            version_file_path: path.join(Self::VERSION_FILENAME),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Option<Self>, io::Error> {
        if let Some(version_file_path) = util::locate_config_file(path, Self::VERSION_FILENAME)? {
            Ok(Some(Self {
                root_path: version_file_path.parent().unwrap().to_path_buf(),
                version_file_path,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn from_current_dir() -> Result<Option<Self>, io::Error> {
        let current_dir = env::current_dir().unwrap();

        Self::from_path(current_dir)
    }

    pub fn exists(&self) -> bool {
        self.version_file_path.exists()
    }

    pub fn get_root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn get_version_file_path(&self) -> &Path {
        &self.version_file_path
    }

    pub fn get_version(&self) -> Result<Version, VutError> {
        let version_str = {
            let mut file = util::open_file(&self.version_file_path)
                .map_err(|err| VutError::VersionFileOpen(err))?;

            let mut version_str = String::new();

            file.read_to_string(&mut version_str)
                .map_err(|err| VutError::VersionFileRead(err))?;

            version_str
        };

        let version = version_str.parse().unwrap();

        Ok(version)
    }

    pub fn set_version(&self, version: &Version) -> Result<(), VutError> {
        let mut file = util::create_file(&self.version_file_path)
            .map_err(|err| VutError::VersionFileOpen(err))?;

        file.write(version.to_string().as_bytes())
            .map_err(|err| VutError::VersionFileWrite(err))?;

        Ok(())
    }

    pub fn bump_version(&self, bump_version: BumpVersion) -> Result<Version, VutError> {
        let version = self.get_version()?;

        let version = match bump_version {
            BumpVersion::Major => version.bump_major(),
            BumpVersion::Minor => version.bump_minor(),
            BumpVersion::Patch => version.bump_patch(),
            BumpVersion::Prerelease => version.bump_prerelease(),
            BumpVersion::Build => version.bump_build(),
        };

        self.set_version(&version)?;

        Ok(version)
    }

    pub fn generate_template_input(&self) -> Result<TemplateInput, VutError> {
        let version = self.get_version()?;

        let mut template_input = TemplateInput::new();

        let split_prerelease = version::split_numbered_prerelease(&version.prerelease);
        let split_build = version::split_numbered_prerelease(&version.build);

        template_input.values.insert("FullVersion".to_owned(), version.to_string());
        template_input.values.insert("Version".to_owned(), Version { build: "".to_owned(), ..version.clone() }.to_string());
        template_input.values.insert("MajorMinorPatch".to_owned(), format!("{}.{}.{}", version.major, version.minor, version.patch));
        template_input.values.insert("MajorMinor".to_owned(), format!("{}.{}", version.major, version.minor));
        template_input.values.insert("Major".to_owned(), format!("{}", version.major));
        template_input.values.insert("Minor".to_owned(), format!("{}", version.minor));
        template_input.values.insert("Patch".to_owned(), format!("{}", version.patch));
        template_input.values.insert("Prerelease".to_owned(), version.prerelease.to_owned());
        template_input.values.insert("PrereleasePrefix".to_owned(), split_prerelease.and_then(|sp| Some(sp.0.to_owned())).unwrap_or_else(|| "".to_owned()));
        template_input.values.insert("PrereleaseNumber".to_owned(), split_prerelease.and_then(|sp| Some(format!("{}", sp.1))).unwrap_or_else(|| "".to_owned()));
        template_input.values.insert("Build".to_owned(), version.build.to_owned());
        template_input.values.insert("BuildPrefix".to_owned(), split_build.and_then(|sp| Some(sp.0.to_owned())).unwrap_or_else(|| "".to_owned()));
        template_input.values.insert("BuildNumber".to_owned(), split_build.and_then(|sp| Some(format!("{}", sp.1))).unwrap_or_else(|| "".to_owned()));

        Ok(template_input)
    }

    pub fn generate_output(&self) -> Result<(), VutError> {
        let root_path = &self.root_path;
        let template_input = self.generate_template_input()?;

        // Set current path to the root path.
        // This is currently required to ensure that relative paths in the configuration
        // are resolved correctly.
        env::set_current_dir(root_path)
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        let files: Vec<PathBuf> = glob::glob("*.vutemplate")
        .expect("No glob!")
        .filter_map(|path| path.ok())
        // Make paths absolute
        .map(|path| util::normalize_path(&path).into_owned())
        // Exclude paths outside the root path
        .filter(|path| path.starts_with(&root_path))
        .collect();

        let mut processed_files: Vec<PathBuf> = Vec::new();
        let mut generated_files: Vec<PathBuf> = Vec::new();

        for file in files {
            let generated_file = template::generate_template::<template::processor::ClassicProcessor>(&file, &template_input, None)
                .map_err(|err| VutError::TemplateGenerate(err))?;

            processed_files.push(file);
            generated_files.push(generated_file);
        }

        Ok(())
    }
}
