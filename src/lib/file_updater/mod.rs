use std::path::Path;

use crate::project::VutError;
use crate::template;

mod custom_regex;

pub use custom_regex::*;

pub trait FileUpdater {
    fn update_file(
        &self,
        path: &Path,
        encoding: Option<&str>,
        template_input: &template::TemplateInput,
    ) -> Result<(), VutError>;
}
