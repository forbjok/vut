use std::borrow::Cow;
use std::fmt;
use std::io::{self, Read, Write};
use std::path::Path;

use encoding::label::encoding_from_whatwg_label;
use encoding::{DecoderTrap, EncoderTrap, EncodingRef};

use crate::util::{self, FileError};

#[derive(Debug)]
pub enum TextFileError {
    Open(FileError),
    Read(io::Error),
    Write(io::Error),
    Encoding(Cow<'static, str>),
    Encode(Cow<'static, str>),
    Decode(Cow<'static, str>),
}

impl fmt::Display for TextFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            TextFileError::Open(err) => write!(f, "{}", err),
            TextFileError::Read(err) => write!(f, "Error reading text file: {}", err),
            TextFileError::Write(err) => write!(f, "Error writing to text file: {}", err),
            TextFileError::Encoding(err) => write!(f, "Invalid encoding: {}", err),
            TextFileError::Encode(err) => write!(f, "Error encoding text file: {}", err),
            TextFileError::Decode(err) => write!(f, "Error decoding text file: {}", err),
        }
    }
}

pub fn read_text_file(file_path: &Path, encoding: Option<&str>) -> Result<String, TextFileError> {
    // If an encoding was specified, try to get an implementation for it.
    let encoding = get_encoding(encoding)?;

    // Open template file.
    let mut file = util::open_file(file_path).map_err(|err| TextFileError::Open(err))?;

    // If an encoding was specified...
    Ok(if let Some(encoding) = encoding {
        // Create a buffer for raw template data.
        let mut buffer: Vec<u8> = Vec::new();

        // Read raw template data into buffer.
        file.read_to_end(&mut buffer).map_err(|err| TextFileError::Read(err))?;

        // Decode the raw data to a string using the specified encoding.
        encoding
            .decode(&buffer, DecoderTrap::Strict)
            .map_err(|err| TextFileError::Decode(err))?
    } else {
        // Create an empty string.
        let mut string: String = String::new();

        // Read template data into the string, assuming it is valid UTF-8.
        file.read_to_string(&mut string)
            .map_err(|err| TextFileError::Read(err))?;

        string
    })
}

pub fn write_text_file(
    file_path: &Path,
    text: impl AsRef<str>,
    encoding: Option<&str>,
) -> Result<usize, TextFileError> {
    // Create file
    let mut file = util::create_file(&file_path).map_err(|err| TextFileError::Open(err))?;

    // Write text to it
    write_text(&mut file, text, encoding)
}

pub fn write_text(
    writable: &mut impl Write,
    text: impl AsRef<str>,
    encoding: Option<&str>,
) -> Result<usize, TextFileError> {
    let text = text.as_ref();

    // If an encoding was specified, try to get an implementation for it.
    let encoding = get_encoding(encoding)?;

    // If an encoding was specified...
    if let Some(encoding) = encoding {
        // Encode the template data using the specified encoding.
        let enc_bytes = encoding
            .encode(&text, EncoderTrap::Strict)
            .map_err(TextFileError::Encode)?;

        // Write bytes
        let bytes_written = writable.write(&enc_bytes).map_err(|err| TextFileError::Write(err))?;

        Ok(bytes_written)
    } else {
        // If no encoding was specified, write the text directly as bytes.
        let bytes_written = writable
            .write(text.as_bytes())
            .map_err(|err| TextFileError::Write(err))?;

        Ok(bytes_written)
    }
}

fn get_encoding(encoding: Option<&str>) -> Result<Option<EncodingRef>, TextFileError> {
    // If an encoding was specified, try to get an implementation for it.
    Ok(match encoding {
        Some(enc_name) => Some(
            encoding_from_whatwg_label(enc_name)
                .ok_or_else(|| TextFileError::Encoding(Cow::Borrowed("Cannot get encoding!")))?,
        ),
        None => None,
    })
}
