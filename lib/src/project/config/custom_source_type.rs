use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type")]
pub enum CustomSourceTypeDef {
    Regex(RegexCustomSourceTypeDef),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RegexCustomSourceTypeDef {
    pub file_name: String,
    pub regex: String,
}
