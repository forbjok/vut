use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use walkdir;

use crate::file_updater::*;
use crate::version::Version;

use super::{config, VutConfig, VutError};

#[derive(Debug)]
struct UpdateFilesSpec {
    include_globset: globset::GlobSet,
    updater_type: String,
    encoding: Option<String>,
}

impl UpdateFilesSpec {
    pub fn from_def(def: &config::UpdateFilesDef) -> Result<Self, VutError> {
        let include_globset = def.globs.build_globset()?;
        let updater_type = def.updater.clone();
        let encoding = def.encoding.clone();

        Ok(Self {
            include_globset,
            updater_type,
            encoding,
        })
    }
}

pub fn update_files(
    config: &VutConfig,
    root_path: &Path,
    version: &Version,
    dir_entries: &[walkdir::DirEntry],
) -> Result<(), VutError> {
    let custom_file_updaters = build_custom_file_updaters(config)?;

    let mut specs: Vec<UpdateFilesSpec> = Vec::new();

    for def in config.update_files.iter() {
        let spec = UpdateFilesSpec::from_def(def)?;

        specs.push(spec);
    }

    for spec in specs.iter() {
        let include_globset = &spec.include_globset;
        let encoding = spec.encoding.as_ref().map(|s| s.as_str());

        // Get the updater for this spec
        let updater = custom_file_updaters.get(&spec.updater_type).ok_or_else(|| {
            VutError::Config(Cow::Owned(format!(
                "File updater type '{}' does not exist!",
                spec.updater_type
            )))
        })?;

        let files_iter = dir_entries
            .iter()
            .map(|entry| entry.path())
            // Only include template files
            .filter_map(|path| {
                // Make path relative, as we only want to match on the path
                // relative to the start path.
                let rel_path = path.strip_prefix(root_path).unwrap();

                if include_globset.is_match(rel_path) {
                    Some(path)
                } else {
                    None
                }
            });

        // Iterate through the files, updating each one.
        for file_path in files_iter {
            updater.update_file(file_path, encoding, version)?;
        }
    }

    Ok(())
}

fn build_custom_file_updaters(config: &VutConfig) -> Result<HashMap<String, Box<dyn FileUpdater>>, VutError> {
    let mut updaters: HashMap<String, Box<dyn FileUpdater>> = HashMap::new();

    for (name, def) in config.file_updaters.iter() {
        match def {
            config::CustomFileUpdaterTypeDef::Regex(def) => {
                let regexes = def.regexes.build_regexes()?;

                let updater = CustomRegexFileUpdater::new(regexes);

                updaters.insert(name.to_owned(), Box::new(updater));
            }
        };
    }

    Ok(updaters)
}
