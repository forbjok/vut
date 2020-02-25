use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Read, Write};
use std::path::Path;

use encoding::label::encoding_from_whatwg_label;
use encoding::{DecoderTrap, EncoderTrap, EncodingRef};
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
    OpenTemplate(util::FileError),
    OpenOutput(util::FileError),
    ReadTemplate(io::Error),
    WriteOutput(io::Error),
    Other(Cow<'static, str>),
}

impl fmt::Display for RenderTemplateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RenderTemplateError::InvalidProcessor(processor_name) => {
                write!(f, "Invalid template processor: {}", processor_name)
            }
            RenderTemplateError::OpenTemplate(err) => write!(f, "Error opening template: {}", err),
            RenderTemplateError::OpenOutput(err) => write!(f, "Error opening output: {}", err),
            RenderTemplateError::ReadTemplate(err) => write!(f, "Error reading template: {}", err),
            RenderTemplateError::WriteOutput(err) => write!(f, "Error writing output: {}", err),
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

    // If an encoding was specified, try to get an implementation for it.
    let encoding: Option<EncodingRef> =
        encoding.map(|enc_name| encoding_from_whatwg_label(&enc_name).expect("Cannot get encoding!"));

    let text = {
        // Open template file.
        let mut file = util::open_file(template_path)
            .map_err::<RenderTemplateError, _>(|err| RenderTemplateError::OpenTemplate(err))?;

        // If an encoding was specified...
        if let Some(encoding) = encoding {
            // Create a buffer for raw template data.
            let mut buffer: Vec<u8> = Vec::new();

            // Read raw template data into buffer.
            file.read_to_end(&mut buffer)
                .map_err::<RenderTemplateError, _>(|err| RenderTemplateError::ReadTemplate(err))?;

            // Decode the raw data to a string using the specified encoding.
            encoding.decode(&buffer, DecoderTrap::Strict).expect("Error decoding!")
        } else {
            // Create an empty string.
            let mut string: String = String::new();

            // Read template data into the string, assuming it is valid UTF-8.
            file.read_to_string(&mut string)
                .map_err::<RenderTemplateError, _>(|err| RenderTemplateError::ReadTemplate(err))?;

            string
        }
    };

    let text = render_template::<TP>(&text, values)?;

    // Create output file.
    let mut output_file = util::create_file(&output_file_path)
        .map_err::<RenderTemplateError, _>(|err| RenderTemplateError::OpenOutput(err))?;

    // If an encoding was specified...
    let output_data: Vec<u8> = if let Some(encoding) = encoding {
        // Encode the template data using the specified encoding.
        encoding
            .encode(&text, EncoderTrap::Strict)
            .map_err(RenderTemplateError::from_string)?
    } else {
        // If no encoding was specified, just convert the string directly into a UTF-8 byte vector.
        text.into_bytes()
    };

    // Write data to output file
    output_file
        .write(&output_data)
        .map_err::<RenderTemplateError, _>(|err| RenderTemplateError::WriteOutput(err))?;

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
