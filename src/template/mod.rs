use std::collections::HashMap;

pub mod processor;

#[derive(Debug)]
pub struct TemplateInput {
    pub values: HashMap<String, String>,
}

impl TemplateInput {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn merge_from(&mut self, other: Self) {
        self.values.extend(other.values);
    }
}
