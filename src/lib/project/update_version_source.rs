use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;

use globset;
use walkdir;

use crate::version::Version;
use crate::version_source::{self, VersionSource, VersionSourceType};

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

enum VersionSourceTemplate {
    Builtin(VersionSourceType),
    CustomRegex(Rc<version_source::CustomRegexSourceTemplate>),
}

impl VersionSourceTemplate {
    pub fn version_source_from_path(&self, path: &Path) -> Option<Box<dyn VersionSource>> {
        match self {
            VersionSourceTemplate::Builtin(vst) => vst.from_path(path),
            VersionSourceTemplate::CustomRegex(template) => template
                .instance_from_path(path)
                .map(|vs| Box::new(vs) as Box<dyn VersionSource>),
        }
    }
}

pub struct CustomSourceTypes {
    regex_source_types: HashMap<String, Rc<version_source::CustomRegexSourceTemplate>>,
}

impl CustomSourceTypes {
    pub fn from_config(config: &VutConfig) -> Result<Self, VutError> {
        let mut regex_source_types: HashMap<String, Rc<version_source::CustomRegexSourceTemplate>> = HashMap::new();

        // Construct custom source type templates
        for (k, v) in config.version_source_types.iter() {
            // Extract regex custom source type information from the enum.
            // Currently regex is the only type implemented.
            let config::CustomSourceTypeDef::Regex(regex_custom_source_type) = v;

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
            regex_source_types.insert(k.clone(), Rc::new(source));
        }

        Ok(Self { regex_source_types })
    }

    pub fn get_template(&self, name: &str) -> Option<Rc<version_source::CustomRegexSourceTemplate>> {
        self.regex_source_types.get(name).cloned()
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
    source_templates: Option<Vec<VersionSourceTemplate>>,
}

impl VersionSourceSpec {
    pub fn from_config_vs(
        def: &config::UpdateVersionSourcesDef,
        custom_source_types: &CustomSourceTypes,
    ) -> Result<Self, VutError> {
        let include_globset = def.globs.build_globset()?;
        let exclude_globset = match &def.exclude_globs {
            Some(ep) => Some(ep.build_globset()?),
            None => None,
        };

        let source_templates = if let Some(vst) = def.types.as_ref() {
            let type_names = &vst.0;

            let mut source_templates: Vec<VersionSourceTemplate> = Vec::with_capacity(type_names.len());

            for name in type_names.iter() {
                // Check for built-in version source type first...
                if let Ok(vst) = VersionSourceType::from_str(&name) {
                    source_templates.push(VersionSourceTemplate::Builtin(vst));
                    continue;
                }

                // ... then check for custom source type.
                if let Some(custom_source_template) = custom_source_types.get_template(&name) {
                    source_templates.push(VersionSourceTemplate::CustomRegex(custom_source_template.clone()));
                    continue;
                }
            }

            Some(source_templates)
        } else {
            None
        };

        Ok(Self {
            include_globset,
            exclude_globset,
            source_templates,
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
            let mut new_sources = if let Some(source_templates) = &self.source_templates {
                // Find built-in sources
                source_templates
                    .iter()
                    .filter_map(|st| st.version_source_from_path(&path))
                    .collect()
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
        let custom_source_types = CustomSourceTypes::from_config(config)?;

        let mut specs: Vec<VersionSourceSpec> = Vec::new();

        for cfg_vs in config.update_version_sources.iter() {
            let spec = VersionSourceSpec::from_config_vs(cfg_vs, &custom_source_types)?;

            specs.push(spec);
        }

        Ok(Self { specs })
    }

    pub fn find_version_sources(&self, path: &Path, rel_path: &Path) -> Vec<Box<dyn VersionSource>> {
        self.specs
            .iter()
            .flat_map(|spec| spec.find_version_sources(path, rel_path).into_iter())
            .collect()
    }
}
