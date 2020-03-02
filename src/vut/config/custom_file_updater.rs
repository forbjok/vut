use serde_derive::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum CustomFileUpdaterTypeDef {
    Regex(RegexFileUpdaterTypeDef),
}

#[derive(Debug, Deserialize)]
pub struct RegexFileUpdaterTypeDef {
    pub regexes: Regexes,
}
