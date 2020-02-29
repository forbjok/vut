use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use log::info;

use crate::util;

pub mod processor;

#[derive(Debug)]
pub struct TemplateInput {
    pub values: HashMap<String, String>,
}

impl TemplateInput {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn merge_from(&mut self, other: Self) {
        self.values.extend(other.values);
    }
}

pub trait TemplateProcessor {
    fn process(template: &str, variables: &TemplateInput) -> Result<String, String>;
}

#[derive(Debug)]
pub enum RenderTemplateError {
    InvalidProcessor(Cow<'static, str>),
    TemplateFile(util::TextFileError),
    OutputFile(util::TextFileError),
    Other(Cow<'static, str>),
}

impl fmt::Display for RenderTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RenderTemplateError::InvalidProcessor(processor_name) => {
                write!(f, "Invalid template processor: {}", processor_name)
            }
            RenderTemplateError::TemplateFile(err) => write!(f, "Template file: {}", err),
            RenderTemplateError::OutputFile(err) => write!(f, "Output file: {}", err),
            RenderTemplateError::Other(err) => write!(f, "{}", err),
        }
    }
}

impl RenderTemplateError {
    pub fn from_string(string: impl Into<Cow<'static, str>>) -> Self {
        RenderTemplateError::Other(string.into())
    }
}

pub fn render_template<TP: TemplateProcessor>(
    text: &str,
    values: &TemplateInput,
) -> Result<String, RenderTemplateError> {
    // Process the template using the specified template processor.
    let text = TP::process(&text, &values).map_err(RenderTemplateError::from_string)?;

    Ok(text)
}

pub fn generate_template<TP: TemplateProcessor>(
    template_path: &Path,
    output_file_path: &Path,
    values: &TemplateInput,
    encoding: Option<&str>,
) -> Result<(), RenderTemplateError> {
    info!("Generating template file {}", template_path.display());

    // Read text from template
    let text = util::read_text_file(template_path, encoding).map_err(RenderTemplateError::TemplateFile)?;

    // Render template
    let text = render_template::<TP>(&text, values)?;

    // Write text to output file
    util::write_text_file(output_file_path, text, encoding).map_err(RenderTemplateError::OutputFile)?;

    Ok(())
}

pub fn generate_template_with_processor_name(
    processor_name: &str,
    template_path: &Path,
    output_file_path: &Path,
    values: &TemplateInput,
    encoding: Option<&str>,
) -> Result<(), RenderTemplateError> {
    match processor_name {
        "vut" => generate_template::<processor::VutProcessor>(template_path, output_file_path, values, encoding),
        _ => Err(RenderTemplateError::InvalidProcessor(Cow::Owned(
            processor_name.to_owned(),
        ))),
    }
}
