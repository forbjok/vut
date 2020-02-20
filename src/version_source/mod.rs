use std::path::Path;

use crate::util;
use crate::version::Version;
use crate::vut::VutError;

mod cargo;
mod npm;
mod version_file;

pub use cargo::*;
pub use npm::*;
pub use version_file::*;

/// Trait representing the authoritative source of a project's version
pub trait VersionSource {
    fn get_path(&self) -> &Path;
    fn exists(&self) -> bool;
    fn get_version(&self) -> Result<Version, VutError>;
    fn set_version(&mut self, version: &Version) -> Result<(), VutError>;
}

pub fn version_source_from_path(path: &Path) -> Option<Box<dyn VersionSource>> {
    let source: Option<Box<dyn VersionSource>> = if let Some(source) = VersionFileSource::from_path(path) {
        Some(Box::new(source))
    } else if let Some(source) = CargoSource::from_path(path) {
        Some(Box::new(source))
    } else if let Some(source) = NpmSource::from_path(path) {
        Some(Box::new(source))
    } else {
        None
    };

    source
}

macro_rules! generate_build_version_source_checker {
    (
        $fn_name:ident {
            $( $src_name:expr => $src_type:ty ),* $(,)*
        }
    ) => {
        pub fn $fn_name(source_names: &[String]) -> Box<dyn Fn(&Path) -> Option<Box<dyn VersionSource>>>
        {
            let mut checkers: Vec<Box<dyn Fn(&Path) -> Option<Box<dyn VersionSource>>>> = Vec::new();

            if source_names.iter().any(|n| n == "*") {
                // Source names contains "*" wildcard - add all version source checks
                $( checkers.push(Box::new(move |path: &Path| <$src_type>::from_path(path).map(|s| Box::new(s) as Box<dyn VersionSource>))); )*
            } else {
                // Source names does not contain wildcard. Add sources based on names in the list.
                for name in source_names {
                    let checker: Option<Box<dyn Fn(&Path) -> Option<Box<dyn VersionSource>>>> = match name.as_str() {
                        $( $src_name => Some(Box::new(move |path: &Path| <$src_type>::from_path(path).map(|s| Box::new(s) as Box<dyn VersionSource>))), )*
                        _ => None,
                    };

                    if let Some(checker) = checker {
                        checkers.push(checker);
                    }
                }
            }

            Box::new(move |path: &Path| -> Option<Box<dyn VersionSource>> {
                for checker in checkers.iter() {
                    if let Some(source) = checker(path) {
                        return Some(source);
                    }
                }

                None
            })
        }
    };
}

generate_build_version_source_checker! {
    build_version_source_checker {
        "vut" => VersionFileSource,
        "cargo" => CargoSource,
        "npm" => NpmSource,
    }
}

pub fn locate_version_source_from(start_path: &Path) -> Option<Box<dyn VersionSource>> {
    util::find_outwards(start_path, |path| version_source_from_path(path)).map(|(_, source)| source)
}
