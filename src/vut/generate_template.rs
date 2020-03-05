use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use globset;
use log::debug;
use walkdir;

use crate::template::{self, TemplateInput};
use crate::version::{self, Version};

use super::{config, VutConfig, VutError};

#[derive(Debug)]
struct TemplateSpec<'a> {
    include_globset: globset::GlobSet,
    def: &'a config::TemplatesDef,
}

impl<'a> TemplateSpec<'a> {
    pub fn from_config_template(def: &'a config::TemplatesDef) -> Result<Self, VutError> {
        let include_globset = def.globs.build_globset()?;

        Ok(Self { include_globset, def })
    }
}

pub fn generate_template_output(
    config: &VutConfig,
    root_path: &Path,
    dir_entries: &[walkdir::DirEntry],
    template_input: &TemplateInput,
) -> Result<(), VutError> {
    let specs = build_template_specs(config)?;

    let mut processed_files: Vec<PathBuf> = Vec::new();
    let mut generated_files: Vec<PathBuf> = Vec::new();

    for spec in specs.iter() {
        debug!("{:?}", spec);

        let include_globset = &spec.include_globset;
        let def = spec.def;

        let start_path: Cow<Path> = match &def.start_path {
            Some(p) => Cow::Owned(root_path.join(p)),
            None => Cow::Borrowed(root_path),
        };

        let output_path = match &def.output_path {
            Some(p) => Cow::Owned(root_path.join(p)),
            None => start_path.clone(),
        };

        let processor = def
            .processor
            .as_ref()
            .map(|pt| pt.to_processor_type())
            .unwrap_or_else(|| template::ProcessorType::Vut);
        let encoding = def.encoding.as_ref().map(|s| s.as_str());

        let template_files_iter = dir_entries
            .iter()
            .map(|entry| entry.path())
            // Exclude files outside the start path
            .filter(|path| path.starts_with(&start_path))
            // Only include template files
            .filter_map(|path| {
                // Make path relative, as we only want to match on the path
                // relative to the start path.
                let rel_path = path.strip_prefix(&start_path).unwrap();

                if include_globset.is_match(rel_path) {
                    Some((path, rel_path))
                } else {
                    None
                }
            });

        for (path, rel_path) in template_files_iter {
            debug!("Processing template {}", path.display());

            let output_file_name: &OsStr = path.file_stem().unwrap();
            let output_file_path = output_path.join(rel_path.with_file_name(output_file_name));

            template::generate_template_with_processor_type(
                &processor,
                path,
                &output_file_path,
                &template_input,
                encoding,
            )
            .map_err(|err| VutError::TemplateGenerate(err))?;

            processed_files.push(path.to_path_buf());
            generated_files.push(output_file_path);
        }
    }

    Ok(())
}

pub fn generate_template_input(version: &Version) -> Result<TemplateInput, VutError> {
    let mut template_input = TemplateInput::new();
    let values = &mut template_input.values;

    let split_prerelease = version
        .prerelease
        .as_ref()
        .map_or(None, |p| version::split_numbered_prerelease(p));
    let split_build = version
        .build
        .as_ref()
        .map_or(None, |b| version::split_numbered_prerelease(b));

    values.insert("FullVersion".to_owned(), version.to_string());
    values.insert(
        "Version".to_owned(),
        Version {
            build: None,
            ..version.clone()
        }
        .to_string(),
    );
    values.insert(
        "MajorMinorPatch".to_owned(),
        format!("{}.{}.{}", version.major, version.minor, version.patch),
    );
    values.insert("MajorMinor".to_owned(), format!("{}.{}", version.major, version.minor));
    values.insert("Major".to_owned(), format!("{}", version.major));
    values.insert("Minor".to_owned(), format!("{}", version.minor));
    values.insert("Patch".to_owned(), format!("{}", version.patch));
    values.insert(
        "Prerelease".to_owned(),
        version.prerelease.as_ref().map_or("", |p| p).to_owned(),
    );
    values.insert(
        "PrereleasePrefix".to_owned(),
        split_prerelease
            .and_then(|sp| Some(sp.0.to_owned()))
            .unwrap_or_else(|| "".to_owned()),
    );
    values.insert(
        "PrereleaseNumber".to_owned(),
        split_prerelease
            .and_then(|sp| Some(format!("{}", sp.1)))
            .unwrap_or_else(|| "".to_owned()),
    );
    values.insert("Build".to_owned(), version.build.as_ref().map_or("", |b| b).to_owned());
    values.insert(
        "BuildPrefix".to_owned(),
        split_build
            .and_then(|sp| Some(sp.0.to_owned()))
            .unwrap_or_else(|| "".to_owned()),
    );
    values.insert(
        "BuildNumber".to_owned(),
        split_build
            .and_then(|sp| Some(format!("{}", sp.1)))
            .unwrap_or_else(|| "".to_owned()),
    );

    Ok(template_input)
}

fn build_template_specs(config: &VutConfig) -> Result<Vec<TemplateSpec>, VutError> {
    let mut specs: Vec<TemplateSpec> = Vec::new();

    for cfg_t in config.templates.iter() {
        let spec = TemplateSpec::from_config_template(cfg_t)?;

        specs.push(spec);
    }

    Ok(specs)
}
