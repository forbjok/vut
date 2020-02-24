use std::collections::HashSet;
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

pub fn first_version_source_from_path(path: &Path) -> Option<Box<dyn VersionSource>> {
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

pub fn locate_first_version_source_from(start_path: &Path) -> Option<Box<dyn VersionSource>> {
    util::find_outwards(start_path, |path| first_version_source_from_path(path)).map(|(_, source)| source)
}

macro_rules! generate_build_version_source_checker {
    (
        $fn_name:ident {
            $( $src_name:expr => $src_type:ty ),* $(,)*
        }
    ) => {
        pub fn $fn_name(source_names: &[String]) -> Box<dyn Fn(&Path) -> Vec<Box<dyn VersionSource>>>
        {
            let source_types: HashSet<String> = source_names.iter().map(|n| n.clone()).collect();

            if source_types.contains("*") {
                Box::new(move |path: &Path| -> Vec<Box<dyn VersionSource>> {
                    let mut sources: Vec<Box<dyn VersionSource>> = Vec::new();

                    $(
                        if let Some(source) = <$src_type>::from_path(path).map(|s| Box::new(s) as Box<dyn VersionSource>) {
                            sources.push(source);
                        }
                    )*

                    sources
                })
            } else {
                Box::new(move |path: &Path| -> Vec<Box<dyn VersionSource>> {
                    let mut sources: Vec<Box<dyn VersionSource>> = Vec::new();

                    $(
                        if source_types.contains($src_name) {
                            if let Some(source) = <$src_type>::from_path(path).map(|s| Box::new(s) as Box<dyn VersionSource>) {
                                sources.push(source);
                            }
                        }
                    )*

                    sources
                })
            }
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
