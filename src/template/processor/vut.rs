use lazy_static::lazy_static;
use regex::Regex;

use crate::template::{TemplateInput, TemplateProcessor};

lazy_static! {
    static ref REGEX_FIND_TEMPLATE_VARS: Regex =
        Regex::new(r#"\{\{(?:\|([^\|]*)\|)?([\w\d]*)(?:\|([^\|]*)\|)?\}\}"#).unwrap();
}

pub struct VutProcessor;

impl TemplateProcessor for VutProcessor {
    fn process(template: &str, values: &TemplateInput) -> Result<String, String> {
        let variables = &values.values;

        let mut variables_not_found: Vec<String> = Vec::new();

        let output = REGEX_FIND_TEMPLATE_VARS
            .replace_all(&template, |captures: &regex::Captures| {
                let prefix = captures.get(1).map(|v| v.as_str()).unwrap_or("");
                let variable_name = &captures[2];
                let suffix = captures.get(3).map(|v| v.as_str()).unwrap_or("");

                let variable_value = if let Some(value) = variables.get(variable_name) {
                    value
                } else {
                    variables_not_found.push(variable_name.to_owned());

                    ""
                };

                if variable_value.is_empty() {
                    // If variable is empty, return a blank string.
                    "".to_owned()
                } else {
                    // If variable is not empty, concatenate prefix, value and suffix
                    format!("{}{}{}", prefix, variable_value, suffix)
                }
            })
            .into_owned();

        if !variables_not_found.is_empty() {
            return Err(format!("Variables not found: {}", variables_not_found.join(", ")));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    fn make_values() -> TemplateInput {
        let mut variables: HashMap<String, String> = HashMap::new();

        variables.insert("TheVariable".to_owned(), "42".to_owned());
        variables.insert("EmptyVariable".to_owned(), "".to_owned());

        TemplateInput { values: variables }
    }

    macro_rules! test_processor {
        (
            $test_name:ident : $processor:ident {
                ok {
                    $( $template_data:expr => $expected_output:expr )*
                }

                err {
                    $( $err_template_data:expr => $err_expected_output:expr )*
                }
            }
        ) => {
            #[test]
            fn $test_name() {
                let values = make_values();

                $( assert_eq!($processor::process($template_data, &values).unwrap(), $expected_output); )*

                $( assert_eq!($processor::process($err_template_data, &values), Err($err_expected_output.to_owned())); )*
            }
        };
    }

    test_processor! {
        test_classic : VutProcessor {
            ok {
                "BLAH={{TheVariable}};" => "BLAH=42;"
                "BLAH={{|.|TheVariable}};" => "BLAH=.42;"
                "BLAH={{|.|TheVariable|.|}};" => "BLAH=.42.;"
                "BLAH={{TheVariable|.|}};" => "BLAH=42.;"
                "BLAH={{EmptyVariable}};" => "BLAH=;"
                "BLAH={{|.|EmptyVariable}};" => "BLAH=;"
                "BLAH={{|.|EmptyVariable|.|}};" => "BLAH=;"
                "BLAH={{EmptyVariable|.|}};" => "BLAH=;"
                "BLAH={{TheVariable}};YADA={{EmptyVariable}};" => "BLAH=42;YADA=;"
                "BLAH={{TheVariable}}.{{TheVariable}}.{{TheVariable}};" => "BLAH=42.42.42;"
                "BLAH={{TheVariable}}.{{TheVariable}}.{{TheVariable}}{{|-|TheVariable}}{{|+|TheVariable}};" => "BLAH=42.42.42-42+42;"
                "BLAH={{|prefix|TheVariable}};" => "BLAH=prefix42;"
                "BLAH={{TheVariable|suffix|}};" => "BLAH=42suffix;"
            }

            err {
                "BLAH={{NonExistentVariable}};" => "Variables not found: NonExistentVariable"
            }
        }
    }
}
