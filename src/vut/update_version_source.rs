use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::Path;

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
    let version_sources_globsets = build_version_sources_globsets(config)?;
    let custom_source_types = build_custom_source_type_templates(config)?;

    let mut sources: Vec<Box<dyn VersionSource>> = Vec::new();

    let dirs_iter = dir_entries
        .iter()
        .map(|entry| entry.path())
        // Only include directories
        .filter(|path| path.is_dir());

    for path in dirs_iter {
        // Make path relative, as we only want to match on the path
        // relative to the root.
        let rel_path = path.strip_prefix(root_path).unwrap();

        for (include_globset, exclude_globset, source_types) in version_sources_globsets.iter() {
            if include_globset.is_match(&rel_path) {
                if let Some(exclude_globset) = exclude_globset {
                    if exclude_globset.is_match(&rel_path) {
                        continue;
                    }
                }

                // Check for built-in version sources at this path
                let mut new_sources = if let Some(source_types) = source_types {
                    let new_sources = version_source::specific_version_sources_from_path(&path, &source_types);

                    for source_type in source_types {
                        if let Some(custom_source_type_template) = custom_source_types.get(source_type.as_str()) {
                            if let Some(source) = custom_source_type_template.instance_from_path(&path) {
                                sources.push(Box::new(source));
                            }
                        }
                    }

                    new_sources
                } else {
                    version_source::version_sources_from_path(&path)
                };

                // Append all found sources to the main list of sources
                sources.append(&mut new_sources);
            }
        }
    }

    for mut source in sources {
        source.set_version(&version)?;
    }

    Ok(())
}

pub fn build_custom_source_type_templates(
    config: &VutConfig,
) -> Result<HashMap<String, version_source::CustomRegexSourceTemplate>, VutError> {
    let mut custom_source_types: HashMap<String, version_source::CustomRegexSourceTemplate> = HashMap::new();

    // Construct custom source type templates
    for (k, v) in config.custom_source_types.iter() {
        // Extract regex custom source type information from the enum.
        // Currently regex is the only type implemented.
        let regex_custom_source_type = match v {
            config::CustomSourceType::Regex(v) => v,
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
        custom_source_types.insert(k.clone(), source);
    }

    Ok(custom_source_types)
}

/// Build a GlobSet from the update_sources patterns in the configuration
fn build_version_sources_globsets(
    config: &VutConfig,
) -> Result<Vec<(globset::GlobSet, Option<globset::GlobSet>, Option<HashSet<String>>)>, VutError> {
    let mut update_version_sources: Vec<(globset::GlobSet, Option<globset::GlobSet>, Option<HashSet<String>>)> =
        Vec::new();

    for version_source in config.version_source.iter() {
        let (pattern, exclude_pattern, source_types) = match version_source {
            config::VersionSource::Simple(pattern) => (pattern, None, None),
            config::VersionSource::Detailed(vs) => (&vs.pattern, vs.exclude_pattern.as_ref(), vs.types.as_ref()),
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

        update_version_sources.push((include_globset, exclude_globset, source_types.map(|v| v.clone())));
    }

    Ok(update_version_sources)
}
