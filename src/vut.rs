use std::env;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use strum_macros::EnumString;

use crate::template::TemplateInput;
use crate::util;
use crate::version::Version;

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
    TemplateGenerate,
}

pub struct Vut {
    root_path: PathBuf,
    version_file_path: PathBuf,
}

impl Vut {
    const VERSION_FILENAME: &'static str = "VERSION";

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

    fn generate_vut_template_input(version: &str) -> Result<TemplateInput, String> {
        let mut template_input = TemplateInput::new();

        template_input.values.insert("FullVersion".to_owned(), version.to_owned());

        Ok(template_input)
    }
}
