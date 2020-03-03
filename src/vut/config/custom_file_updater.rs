use std::convert::{TryFrom, TryInto};

use serde_derive::Deserialize;

use crate::file_updater::*;
use crate::vut::VutError;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum CustomFileUpdaterTypeDef {
    Regex(RegexFileUpdaterTypeDef),
}

#[derive(Debug, Deserialize)]
pub struct RegexReplacerDef {
    pub regexes: Regexes,
    pub template: Option<String>,
    pub template_processor: Option<TemplateProcessorType>,
}

#[derive(Debug, Deserialize)]
pub struct RegexFileUpdaterTypeDef {
    pub replacers: Vec<RegexReplacerDef>,
}

impl TryFrom<&RegexReplacerDef> for RegexReplacer {
    type Error = VutError;

    fn try_from(def: &RegexReplacerDef) -> Result<Self, Self::Error> {
        Ok(Self {
            regexes: def.regexes.build_regexes()?,
            template: def.template.as_ref().map(|s| s.clone()),
            template_processor: def.template_processor.as_ref().map(|pt| pt.to_processor_type()),
        })
    }
}

impl TryFrom<&RegexFileUpdaterTypeDef> for CustomRegexFileUpdater {
    type Error = VutError;

    fn try_from(def: &RegexFileUpdaterTypeDef) -> Result<Self, Self::Error> {
        let mut replacers: Vec<RegexReplacer> = Vec::new();

        for replacer_def in def.replacers.iter() {
            replacers.push(replacer_def.try_into()?);
        }

        Ok(Self::new(replacers))
    }
}
