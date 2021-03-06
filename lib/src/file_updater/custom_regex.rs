use std::borrow::Cow;
use std::path::Path;

use regex::Regex;

use crate::project::VutError;
use crate::template;
use crate::util;

use super::FileUpdater;

pub struct RegexReplacer {
    pub regexes: Vec<Regex>,
    pub template: Option<String>,
    pub template_processor: Option<template::ProcessorType>,
}

pub struct CustomRegexFileUpdater {
    replacers: Vec<RegexReplacer>,
}

impl CustomRegexFileUpdater {
    pub fn new(replacers: Vec<RegexReplacer>) -> Self {
        Self { replacers }
    }
}

impl FileUpdater for CustomRegexFileUpdater {
    fn update_file(
        &self,
        file_path: &Path,
        encoding: Option<&str>,
        template_input: &template::TemplateInput,
    ) -> Result<(), VutError> {
        // Read text from file
        let mut text =
            util::read_text_file(file_path, encoding).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        let version_str = template_input.values.get("FullVersion").ok_or_else(|| {
            VutError::Other("FullVersion not found in template input! This is almost certainly a bug.".into())
        })?;

        // Iterate through all regexes, performing replacements for each one.
        for replacer in self.replacers.iter() {
            let replace_with: Cow<str> = if let Some(template) = &replacer.template {
                let template_processor = replacer
                    .template_processor
                    .as_ref()
                    .cloned()
                    .unwrap_or(template::ProcessorType::Vut);

                template::render_template_with_processor_type(&template_processor, template, template_input)
                    .map_err(VutError::TemplateGenerate)?
            } else {
                Cow::Borrowed(version_str)
            };

            for regex in replacer.regexes.iter() {
                text = regex
                    .replace_all(&text, |caps: &regex::Captures| {
                        format!("{}{}{}", &caps[1], &replace_with, &caps[3])
                    })
                    .into_owned();
            }
        }

        // Write updated text to file
        util::write_text_file(file_path, text, encoding).map_err(|err| VutError::Other(Cow::Owned(err.to_string())))?;

        Ok(())
    }
}
