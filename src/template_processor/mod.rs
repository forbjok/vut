use crate::template::TemplateInput;

mod classic;

pub use classic::ClassicProcessor;

pub trait TemplateProcessor {
    fn process(template: &str, variables: &TemplateInput) -> Result<String, String>;
}
