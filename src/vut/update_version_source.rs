use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

use globset;
use walkdir;

use crate::version::Version;
use crate::version_source::{self, VersionSource};

use super::{config, VutConfig, VutError};

pub fn update_version_sources(
    config: &VutConfig,
    root_path: &Path,
    version: &Version,
    dir_entries: &[walkdir::DirEntry],
) -> Result<(), VutError> {
    let version_source_finder = VersionSourceFinder::from_config(config)?;

    let mut version_sources: Vec<Box<dyn VersionSource>> = Vec::new();

    let dirs_iter = dir_entries
        .iter()
        .map(|entry| entry.path())
        // Only include directories
        .filter(|path| path.is_dir());

    for path in dirs_iter {
        // Make path relative, as we only want to match on the path
        // relative to the root.
        let rel_path = path.strip_prefix(root_path).unwrap();

        version_sources.append(&mut version_source_finder.find_version_sources(path, rel_path));
    }

    for mut vs in version_sources {
        vs.set_version(&version)?;
    }

    Ok(())
}

pub struct CustomSourceTypes {
    regex_source_types: HashMap<String, version_source::CustomRegexSourceTemplate>,
}

impl CustomSourceTypes {
    pub fn from_config(config: &VutConfig) -> Result<Self, VutError> {
        let mut regex_source_types: HashMap<String, version_source::CustomRegexSourceTemplate> = HashMap::new();

        // Construct custom source type templates
        for (k, v) in config.custom_source_types.iter() {
            // Extract regex custom source type information from the enum.
            // Currently regex is the only type implemented.
            let regex_custom_source_type = match v {
                config::CustomSourceTypeDef::Regex(v) => v,
            };

            // Try to parse regex string
            let regex = {
                let mut builder = regex::RegexBuilder::new(&regex_custom_source_type.regex);
                builder.multi_line(true);

                match builder.build() {
                    Ok(v) => v,
                    Err(err) => {
                        return Err(VutError::Other(Cow::Owned(format!(
                            "Invalid regex '{}': {}",
                            &regex_custom_source_type.regex,
                            err.to_string()
                        ))))
                    }
                }
            };

            // Construct source type template
            let source = version_source::CustomRegexSourceTemplate::new(&regex_custom_source_type.file_name, regex);

            // Add source to hashmap for later use
            regex_source_types.insert(k.clone(), source);
        }

        Ok(Self { regex_source_types })
    }

    pub fn version_sources_from_path(
        &self,
        path: &Path,
        source_types: &HashSet<String>,
    ) -> Vec<Box<dyn VersionSource>> {
        let mut version_sources: Vec<Box<dyn VersionSource>> = Vec::new();

        for source_type in source_types {
            if let Some(custom_source_type_template) = self.regex_source_types.get(source_type.as_str()) {
                if let Some(source) = custom_source_type_template.instance_from_path(&path) {
                    version_sources.push(Box::new(source));
                }
            }
        }

        version_sources
    }
}

struct VersionSourceSpec {
    include_globset: globset::GlobSet,
    exclude_globset: Option<globset::GlobSet>,
    source_types: Option<HashSet<String>>,
    custom_source_types: Rc<CustomSourceTypes>,
}

impl VersionSourceSpec {
    pub fn from_config_vs(
        def: &config::VersionSourceDef,
        custom_source_types: Rc<CustomSourceTypes>,
    ) -> Result<Self, VutError> {
        let (pattern, exclude_pattern, source_types) = match def {
            config::VersionSourceDef::Simple(pattern) => (pattern, None, None),
            config::VersionSourceDef::Detailed(vs) => (&vs.pattern, vs.exclude_pattern.as_ref(), vs.types.as_ref()),
        };

        let mut include_globset = globset::GlobSetBuilder::new();

        let patterns = match pattern {
            config::Patterns::Single(v) => vec![v],
            config::Patterns::Multiple(v) => v.iter().collect(),
        };

        for pattern in patterns {
            let glob = globset::Glob::new(&pattern).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            include_globset.add(glob);
        }

        let include_globset = include_globset
            .build()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        let exclude_globset = if let Some(pattern) = exclude_pattern {
            let patterns = match &pattern {
                config::Patterns::Single(v) => vec![v],
                config::Patterns::Multiple(v) => v.iter().collect(),
            };

            let mut exclude_globset = globset::GlobSetBuilder::new();

            for pattern in patterns {
                let glob = globset::Glob::new(&pattern).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

                exclude_globset.add(glob);
            }

            let exclude_globset = exclude_globset
                .build()
                .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

            Some(exclude_globset)
        } else {
            None
        };

        let source_types = source_types.map(|v| match v {
            config::VersionSourceTypes::Single(name) => {
                let mut set = HashSet::new();
                set.insert(name.clone());

                set
            }
            config::VersionSourceTypes::Multiple(set) => set.clone(),
        });

        Ok(Self {
            include_globset,
            exclude_globset,
            source_types: source_types.map(|v| v.clone()),
            custom_source_types,
        })
    }

    pub fn find_version_sources(&self, path: &Path, rel_path: &Path) -> Vec<Box<dyn VersionSource>> {
        let mut version_sources = Vec::new();

        if self.include_globset.is_match(&rel_path) {
            if let Some(exclude_globset) = &self.exclude_globset {
                if exclude_globset.is_match(&rel_path) {
                    // If path is matched by the exclude globset, immediately return the empty list.
                    return version_sources;
                }
            }

            // Check for built-in version sources at this path
            let mut new_sources = if let Some(source_types) = &self.source_types {
                // Find built-in sources
                let mut new_sources = version_source::specific_version_sources_from_path(path, source_types);

                // Find custom sources
                new_sources.append(&mut self.custom_source_types.version_sources_from_path(path, source_types));

                new_sources
            } else {
                version_source::version_sources_from_path(&path)
            };

            // Append all found sources to the main list of sources
            version_sources.append(&mut new_sources);
        }

        version_sources
    }
}

pub struct VersionSourceFinder {
    specs: Vec<VersionSourceSpec>,
}

impl VersionSourceFinder {
    pub fn from_config(config: &VutConfig) -> Result<Self, VutError> {
        let custom_source_types = Rc::new(CustomSourceTypes::from_config(config)?);

        let mut specs: Vec<VersionSourceSpec> = Vec::new();

        for cfg_vs in config.version_source.iter() {
            let spec = VersionSourceSpec::from_config_vs(cfg_vs, custom_source_types.clone())?;

            specs.push(spec);
        }

        Ok(Self { specs })
    }

    pub fn find_version_sources(&self, path: &Path, rel_path: &Path) -> Vec<Box<dyn VersionSource>> {
        let version_sources = self
            .specs
            .iter()
            .flat_map(|spec| spec.find_version_sources(path, rel_path).into_iter())
            .collect();

        version_sources
    }
}
