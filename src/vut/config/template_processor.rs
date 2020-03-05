use serde_derive::Deserialize;

use crate::template::ProcessorType;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemplateProcessorType {
    Vut,
}

impl TemplateProcessorType {
    pub fn to_processor_type(&self) -> ProcessorType {
        match self {
            Self::Vut => ProcessorType::Vut,
        }
    }
}
