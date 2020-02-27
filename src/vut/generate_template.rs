use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use globset;
use log::debug;
use walkdir;

use crate::template::{self, TemplateInput};
use crate::version::{self, Version};

use super::{config, VutConfig, VutError};

pub fn generate_template_output(
    config: &VutConfig,
    root_path: &Path,
    version: &Version,
    dir_entries: &[walkdir::DirEntry],
) -> Result<(), VutError> {
    let template_globsets = build_template_globsets(config)?;

    let template_input = generate_template_input(version)?;

    let mut processed_files: Vec<PathBuf> = Vec::new();
    let mut generated_files: Vec<PathBuf> = Vec::new();

    for (template, globset) in template_globsets {
        debug!("{:?}", template);

        let start_path: Cow<Path> = match &template.start_path {
            Some(p) => Cow::Owned(root_path.join(p)),
            None => Cow::Borrowed(root_path),
        };

        let output_path = match &template.output_path {
            Some(p) => Cow::Owned(root_path.join(p)),
            None => start_path.clone(),
        };

        let processor = template.processor.as_ref().map(|s| s.as_str()).unwrap_or("vut");
        let encoding = template.encoding.as_ref().map(|s| s.as_str());

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

                if globset.is_match(rel_path) {
                    Some((path, rel_path))
                } else {
                    None
                }
            });

        for (path, rel_path) in template_files_iter {
            debug!("Processing template {}", path.display());

            let output_file_name: &OsStr = path.file_stem().unwrap();
            let output_file_path = output_path.join(rel_path.with_file_name(output_file_name));

            template::generate_template_with_processor_name(
                processor,
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

    let split_prerelease = version
        .prerelease
        .as_ref()
        .map_or(None, |p| version::split_numbered_prerelease(p));
    let split_build = version
        .build
        .as_ref()
        .map_or(None, |b| version::split_numbered_prerelease(b));

    template_input
        .values
        .insert("FullVersion".to_owned(), version.to_string());
    template_input.values.insert(
        "Version".to_owned(),
        Version {
            build: None,
            ..version.clone()
        }
        .to_string(),
    );
    template_input.values.insert(
        "MajorMinorPatch".to_owned(),
        format!("{}.{}.{}", version.major, version.minor, version.patch),
    );
    template_input
        .values
        .insert("MajorMinor".to_owned(), format!("{}.{}", version.major, version.minor));
    template_input
        .values
        .insert("Major".to_owned(), format!("{}", version.major));
    template_input
        .values
        .insert("Minor".to_owned(), format!("{}", version.minor));
    template_input
        .values
        .insert("Patch".to_owned(), format!("{}", version.patch));
    template_input.values.insert(
        "Prerelease".to_owned(),
        version.prerelease.as_ref().map_or("", |p| p).to_owned(),
    );
    template_input.values.insert(
        "PrereleasePrefix".to_owned(),
        split_prerelease
            .and_then(|sp| Some(sp.0.to_owned()))
            .unwrap_or_else(|| "".to_owned()),
    );
    template_input.values.insert(
        "PrereleaseNumber".to_owned(),
        split_prerelease
            .and_then(|sp| Some(format!("{}", sp.1)))
            .unwrap_or_else(|| "".to_owned()),
    );
    template_input
        .values
        .insert("Build".to_owned(), version.build.as_ref().map_or("", |b| b).to_owned());
    template_input.values.insert(
        "BuildPrefix".to_owned(),
        split_build
            .and_then(|sp| Some(sp.0.to_owned()))
            .unwrap_or_else(|| "".to_owned()),
    );
    template_input.values.insert(
        "BuildNumber".to_owned(),
        split_build
            .and_then(|sp| Some(format!("{}", sp.1)))
            .unwrap_or_else(|| "".to_owned()),
    );

    Ok(template_input)
}

pub fn build_template_globsets(config: &VutConfig) -> Result<Vec<(&config::Template, globset::GlobSet)>, VutError> {
    let mut template_globsets: Vec<(&config::Template, globset::GlobSet)> = Vec::new();

    for template in config.template.iter() {
        let patterns = match &template.pattern {
            config::Patterns::Single(v) => vec![v],
            config::Patterns::Multiple(v) => v.iter().collect(),
        };

        let mut globset = globset::GlobSetBuilder::new();

        for pattern in patterns {
            let glob = globset::Glob::new(&pattern).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;
            globset.add(glob);
        }

        let globset = globset
            .build()
            .map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        template_globsets.push((&template, globset));
    }

    Ok(template_globsets)
}
