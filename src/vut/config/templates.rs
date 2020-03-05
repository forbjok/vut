use serde_derive::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TemplatesDef {
    pub globs: Globs,
    pub start_path: Option<PathBuf>,
    pub output_path: Option<PathBuf>,
    pub processor: Option<TemplateProcessorType>,
    pub encoding: Option<String>,
}
