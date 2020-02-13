use std::collections::HashMap;
use std::borrow::Cow;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use encoding::{DecoderTrap, EncodingRef, EncoderTrap};
use encoding::label::encoding_from_whatwg_label;

use crate::util;

pub mod processor;

#[derive(Debug)]
pub struct TemplateInput {
    pub values: HashMap<String, String>,
}

impl TemplateInput {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn merge_from(&mut self, other: Self) {
        self.values.extend(other.values);
    }
}

pub trait TemplateProcessor {
    fn process(template: &str, variables: &TemplateInput) -> Result<String, String>;
}

#[derive(Debug)]
pub struct RenderTemplateError {
    pub description: Cow<'static, str>,
}

impl RenderTemplateError {
    pub fn from_string(string: impl Into<Cow<'static, str>>) -> Self {
        RenderTemplateError {
            description: string.into(),
        }
    }
}

impl From<io::Error> for RenderTemplateError {
    fn from(error: io::Error) -> Self {
        RenderTemplateError {
            description: Cow::Owned(error.to_string()),
        }
    }
}

impl From<util::FileError> for RenderTemplateError {
    fn from(error: util::FileError) -> Self {
        Self {
            description: Cow::Owned(error.to_string()),
        }
    }
}

pub fn render_template<TP: TemplateProcessor>(text: &str, values: &TemplateInput) -> Result<String, RenderTemplateError> {
    // Process the template using the specified template processor.
    let text = TP::process(&text, &values).map_err(RenderTemplateError::from_string)?;

    Ok(text)
}

pub fn generate_template<TP: TemplateProcessor>(template_path: &Path, values: &TemplateInput, encoding: Option<String>) -> Result<PathBuf, RenderTemplateError> {
    // If an encoding was specified, try to get an implementation for it.
    let encoding: Option<EncodingRef> = encoding.map(|enc_name| encoding_from_whatwg_label(&enc_name).expect("Cannot get encoding!"));

    let text = {
        // Open template file.
        let mut file = util::open_file(template_path)
            .map_err::<RenderTemplateError, _>(|err| err.into())?;

        // If an encoding was specified...
        if let Some(encoding) = encoding {
            // Create a buffer for raw template data.
            let mut buffer: Vec<u8> = Vec::new();

            // Read raw template data into buffer.
            file.read_to_end(&mut buffer)
                .map_err::<RenderTemplateError, _>(|err| err.into())?;

            // Decode the raw data to a string using the specified encoding.
            encoding.decode(&buffer, DecoderTrap::Strict).expect("Error decoding!")
        } else {
            // Create an empty string.
            let mut string: String = String::new();

            // Read template data into the string, assuming it is valid UTF-8.
            file.read_to_string(&mut string)
                .map_err::<RenderTemplateError, _>(|err| err.into())?;

            string
        }
    };

    let text = render_template::<TP>(&text, values)?;

    let template_path = util::normalize_path(template_path);

    dbg!(&template_path);

    // Get template directory path
    let template_dir = template_path.parent().unwrap();

    // Get template filename
    let template_filename = template_path.file_stem().unwrap();

    // Construct output file path.
    let output_file_path = template_dir.join(template_filename);

    // Create output file.
    let mut output_file = fs::File::create(&output_file_path)
        .map_err::<RenderTemplateError, _>(|err| err.into())?;

    // If an encoding was specified...
    let output_data: Vec<u8> = if let Some(encoding) = encoding {
        // Encode the template data using the specified encoding.
        encoding.encode(&text, EncoderTrap::Strict)
            .map_err(RenderTemplateError::from_string)?
    } else {
        // If no encoding was specified, just convert the string directly into a UTF-8 byte vector.
        text.into_bytes()
    };

    // Write data to output file
    output_file.write(&output_data)
        .map_err::<RenderTemplateError, _>(|err| err.into())?;

    // Return path to output file
    Ok(output_file_path)
}
