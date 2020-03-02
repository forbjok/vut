use std::borrow::Cow;
use std::path::Path;

use regex::Regex;

use crate::template;
use crate::util;
use crate::vut::VutError;

use super::FileUpdater;

pub struct RegexReplacer {
    pub regexes: Vec<Regex>,
    pub template: String,
    pub template_processor: Option<String>,
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

        // Iterate through all regexes, performing replacements for each one.
        for replacer in self.replacers.iter() {
            let template = &replacer.template;
            let template_processor = replacer
                .template_processor
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_else(|| "vut");

            let replace_with =
                template::render_template_with_processor_name(template_processor, template, template_input)
                    .map_err(|err| VutError::TemplateGenerate(err))?;

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
