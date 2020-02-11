use crate::template::TemplateInput;

pub fn generate_vut_template_input(version: &str) -> Result<TemplateInput, String> {
    let mut template_input = TemplateInput::new();

    template_input.values.insert("FullVersion".to_owned(), version.to_owned());

    Ok(template_input)
}
