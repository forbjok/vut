use serde_derive::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
pub struct TemplatesDef {
    pub globs: Globs,
    pub start_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub processor: Option<TemplateProcessorType>,
    pub encoding: Option<String>,
}
