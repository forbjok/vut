use serde_derive::Deserialize;

use super::*;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum UpdateVersionSourcesDef {
    Simple(Globs),
    Detailed(UpdateVersionSourcesDetail),
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateVersionSourcesDetail {
    pub globs: Globs,
    pub exclude_globs: Option<Globs>,
    pub types: Option<VersionSourceTypes>,
}

impl UpdateVersionSourcesDef {
    pub fn to_detail(&self) -> Cow<UpdateVersionSourcesDetail> {
        match self {
            Self::Simple(globs) => Cow::Owned(UpdateVersionSourcesDetail {
                globs: globs.clone(),
                exclude_globs: None,
                types: None,
            }),
            Self::Detailed(detail) => Cow::Borrowed(detail),
        }
    }
}
